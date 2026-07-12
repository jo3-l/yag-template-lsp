//! Declarative document rules for template actions and clauses.

use yag_template_syntax::ast::{
    Action, AstNode, AstToken, ElseBranch, ElseClause, EndClause, Expr, IfAction, LeftDelim, RangeClause, RangeLoop,
    RightDelim, TemplateBlock, TemplateDefinition, TryCatchAction, WhileLoop, WithAction,
};

use crate::LayoutKind;
use crate::lower::{AllowCompact, Formatter};
use crate::pretty::{Doc, concat, empty, join, soft_line, text, try_concat};

impl Formatter<'_> {
    /// Format an action atomically. A rule that cannot construct a complete
    /// document leaves the original typed action untouched.
    pub(crate) fn action(&mut self, action: Action) -> Doc {
        self.action_rule(&action)
            .unwrap_or_else(|| text(action.syntax().text().to_owned()))
    }

    fn action_rule(&mut self, action: &Action) -> Option<Doc> {
        match action {
            Action::Comment(action) => Some(text(action.syntax().text().to_owned())),
            Action::TemplateDefinition(action) => self.template_definition(action),
            Action::TemplateBlock(action) => self.template_block(action),
            Action::TemplateInvocation(action) => self.template_action(
                action.delims()?,
                "template",
                action.template_name()?.syntax().text().to_string(),
                action.context_data(),
            ),
            Action::Return(action) => self.keyword_with_expression(action.delims()?, "return", action.expr()?),
            Action::If(action) => self.if_action(action),
            Action::With(action) => self.with_action(action),
            Action::Range(action) => self.range_action(action),
            Action::While(action) => self.while_action(action),
            Action::Break(action) => self.keyword(action.delims()?, "break"),
            Action::Continue(action) => self.keyword(action.delims()?, "continue"),
            Action::TryCatch(action) => self.try_catch_action(action),
            Action::ExprAction(action) => {
                let expression = action.expr()?;
                let break_before_close = self.is_top_level_key_value_call(&expression);
                let body = self.expr(expression)?;
                self.delimited_with_breaking_close(action.delims()?, body, break_before_close)
            }
        }
    }

    /// A key/value call is formatted as rows. When it is the whole action,
    /// keep the action delimiter out of the final row without affecting
    /// parenthesized or otherwise nested calls.
    fn is_top_level_key_value_call(&self, expression: &Expr) -> bool {
        let Expr::FuncCall(call) = expression else {
            return false;
        };
        let Some(name) = call.func_name() else {
            return false;
        };
        self.function_layout(name.get()) == Some(LayoutKind::KeyValuePairs) && call.args().count().is_multiple_of(2)
    }

    fn template_definition(&mut self, action: &TemplateDefinition) -> Option<Doc> {
        let allow_compact = allows_compact(action);
        let clause = action.clause()?;
        Some(
            concat([
                self.template_action(
                    clause.delims()?,
                    "define",
                    clause.template_name()?.syntax().text().to_string(),
                    None,
                )?,
                self.body(action.template_body()?, allow_compact),
                self.end_clause(&action.end_clause()?)?,
            ])
            .group_if(allow_compact.is_allowed()),
        )
    }

    fn template_block(&mut self, action: &TemplateBlock) -> Option<Doc> {
        let allow_compact = allows_compact(action);
        let clause = action.clause()?;
        Some(
            concat([
                self.template_action(
                    clause.delims()?,
                    "block",
                    clause.template_name()?.syntax().text().to_string(),
                    clause.context_data(),
                )?,
                self.body(action.template_body()?, allow_compact),
                self.end_clause(&action.end_clause()?)?,
            ])
            .group_if(allow_compact.is_allowed()),
        )
    }

    fn if_action(&mut self, action: &IfAction) -> Option<Doc> {
        let allow_compact = allows_compact(action);
        let clause = action.clause()?;
        Some(
            concat([
                self.keyword_with_expression(clause.delims()?, "if", clause.condition()?)?,
                self.body(action.body()?, allow_compact),
                self.else_branches(action.else_branches(), allow_compact)?,
                self.end_clause(&action.end_clause()?)?,
            ])
            .group_if(allow_compact.is_allowed()),
        )
    }

    fn with_action(&mut self, action: &WithAction) -> Option<Doc> {
        let allow_compact = allows_compact(action);
        let clause = action.clause()?;
        Some(
            concat([
                self.keyword_with_expression(clause.delims()?, "with", clause.condition()?)?,
                self.body(action.body()?, allow_compact),
                self.else_branches(action.else_branches(), allow_compact)?,
                self.end_clause(&action.end_clause()?)?,
            ])
            .group_if(allow_compact.is_allowed()),
        )
    }

    fn else_branches(
        &mut self,
        branches: impl Iterator<Item = ElseBranch>,
        allow_compact: AllowCompact,
    ) -> Option<Doc> {
        try_concat(branches.map(|branch| {
            let clause = branch.clause()?;
            Some(concat([
                self.else_clause(&clause)?,
                self.body(branch.body()?, allow_compact),
            ]))
        }))
    }

    fn else_clause(&mut self, clause: &ElseClause) -> Option<Doc> {
        let condition = if let Some(condition) = clause.condition() {
            concat([text(" "), text("if"), self.expression_argument(condition)?])
        } else {
            empty()
        };
        self.delimited(clause.delims()?, concat([text("else"), condition]))
    }

    fn range_action(&mut self, action: &RangeLoop) -> Option<Doc> {
        let allow_compact = allows_compact(action);
        let clause = action.clause()?;
        let else_branch = if let Some(branch) = action.else_branch() {
            let clause = branch.clause()?;
            concat([self.else_clause(&clause)?, self.body(branch.body()?, allow_compact)])
        } else {
            empty()
        };
        Some(
            concat([
                self.range_clause(&clause)?,
                self.body(action.body()?, allow_compact),
                else_branch,
                self.end_clause(&action.end_clause()?)?,
            ])
            .group_if(allow_compact.is_allowed()),
        )
    }

    fn range_clause(&mut self, clause: &RangeClause) -> Option<Doc> {
        let mut variables = clause.iteration_vars().peekable();
        let binding = if variables.peek().is_some() {
            let assignment = if clause.declares_vars() {
                text(" :=")
            } else if clause.assigns_vars() {
                text(" =")
            } else {
                empty()
            };
            concat([
                text(" "),
                join(text(", "), variables.map(|variable| text(variable.name()))),
                assignment,
            ])
        } else {
            empty()
        };
        let expression = self.expression_argument(clause.expr()?)?;
        self.delimited(clause.delims()?, concat([text("range"), binding, expression]))
    }

    fn while_action(&mut self, action: &WhileLoop) -> Option<Doc> {
        let allow_compact = allows_compact(action);
        let clause = action.clause()?;
        let else_branch = if let Some(branch) = action.else_branch() {
            let clause = branch.clause()?;
            concat([self.else_clause(&clause)?, self.body(branch.body()?, allow_compact)])
        } else {
            empty()
        };
        Some(
            concat([
                self.keyword_with_expression(clause.delims()?, "while", clause.condition()?)?,
                self.body(action.body()?, allow_compact),
                else_branch,
                self.end_clause(&action.end_clause()?)?,
            ])
            .group_if(allow_compact.is_allowed()),
        )
    }

    fn try_catch_action(&mut self, action: &TryCatchAction) -> Option<Doc> {
        let allow_compact = allows_compact(action);
        Some(
            concat([
                self.keyword(action.try_clause()?.delims()?, "try")?,
                self.body(action.try_body()?, allow_compact),
                self.keyword(action.catch_clause()?.delims()?, "catch")?,
                self.body(action.catch_body()?, allow_compact),
                self.end_clause(&action.end_clause()?)?,
            ])
            .group_if(allow_compact.is_allowed()),
        )
    }

    fn end_clause(&self, clause: &EndClause) -> Option<Doc> {
        self.keyword(clause.delims()?, "end")
    }

    fn template_action(
        &mut self,
        delims: (LeftDelim, RightDelim),
        keyword: &str,
        template_name: String,
        context_data: Option<Expr>,
    ) -> Option<Doc> {
        let context_data = if let Some(context_data) = context_data {
            self.expression_argument(context_data)?
        } else {
            empty()
        };
        self.delimited(
            delims,
            concat([text(keyword), text(" "), text(template_name), context_data]),
        )
    }

    fn keyword(&self, delims: (LeftDelim, RightDelim), keyword: &str) -> Option<Doc> {
        self.delimited(delims, text(keyword))
    }

    fn keyword_with_expression(
        &mut self,
        delims: (LeftDelim, RightDelim),
        keyword: &str,
        expression: Expr,
    ) -> Option<Doc> {
        let expression = self.expression_argument(expression)?;
        self.delimited(delims, concat([text(keyword), expression]))
    }

    fn expression_argument(&mut self, expression: Expr) -> Option<Doc> {
        let expression = self.expr(expression)?;
        Some(self.continuation(concat([soft_line(), expression])))
    }
}

fn allows_compact(action: &impl AstNode) -> AllowCompact {
    // Existing physical newlines stay structural rather than participating in reflow.
    if action.syntax().text().contains_char('\n') {
        AllowCompact::No
    } else {
        AllowCompact::Yes
    }
}
