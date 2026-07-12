//! Declarative document rules for template actions and clauses.

use yag_template_syntax::ast::{
    Action, AstNode, AstToken, ElseBranch, ElseClause, EndClause, Expr, IfAction, LeftDelim, RangeClause, RangeLoop,
    RightDelim, TemplateBlock, TemplateDefinition, TryCatchAction, WhileLoop, WithAction,
};

use crate::doc::{Doc, concat, empty, join, soft_line, text, try_concat};
use crate::lower::Formatter;

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
                let body = self.expr(action.expr()?)?;
                self.delimited(action.delims()?, body)
            }
        }
    }

    fn template_definition(&mut self, action: &TemplateDefinition) -> Option<Doc> {
        let clause = action.clause()?;
        Some(concat([
            self.template_action(
                clause.delims()?,
                "define",
                clause.template_name()?.syntax().text().to_string(),
                None,
            )?,
            self.body(action.template_body()?),
            self.end_clause(&action.end_clause()?)?,
        ]))
    }

    fn template_block(&mut self, action: &TemplateBlock) -> Option<Doc> {
        let clause = action.clause()?;
        Some(concat([
            self.template_action(
                clause.delims()?,
                "block",
                clause.template_name()?.syntax().text().to_string(),
                clause.context_data(),
            )?,
            self.body(action.template_body()?),
            self.end_clause(&action.end_clause()?)?,
        ]))
    }

    fn if_action(&mut self, action: &IfAction) -> Option<Doc> {
        let clause = action.clause()?;
        Some(concat([
            self.keyword_with_expression(clause.delims()?, "if", clause.condition()?)?,
            self.body(action.body()?),
            self.else_branches(action.else_branches())?,
            self.end_clause(&action.end_clause()?)?,
        ]))
    }

    fn with_action(&mut self, action: &WithAction) -> Option<Doc> {
        let clause = action.clause()?;
        Some(concat([
            self.keyword_with_expression(clause.delims()?, "with", clause.condition()?)?,
            self.body(action.body()?),
            self.else_branches(action.else_branches())?,
            self.end_clause(&action.end_clause()?)?,
        ]))
    }

    fn else_branches(&mut self, branches: impl Iterator<Item = ElseBranch>) -> Option<Doc> {
        try_concat(branches.map(|branch| {
            let clause = branch.clause()?;
            Some(concat([self.else_clause(&clause)?, self.body(branch.body()?)]))
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
        let clause = action.clause()?;
        let else_branch = if let Some(branch) = action.else_branch() {
            let clause = branch.clause()?;
            concat([self.else_clause(&clause)?, self.body(branch.body()?)])
        } else {
            empty()
        };
        Some(concat([
            self.range_clause(&clause)?,
            self.body(action.body()?),
            else_branch,
            self.end_clause(&action.end_clause()?)?,
        ]))
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
        let clause = action.clause()?;
        let else_branch = if let Some(branch) = action.else_branch() {
            let clause = branch.clause()?;
            concat([self.else_clause(&clause)?, self.body(branch.body()?)])
        } else {
            empty()
        };
        Some(concat([
            self.keyword_with_expression(clause.delims()?, "while", clause.condition()?)?,
            self.body(action.body()?),
            else_branch,
            self.end_clause(&action.end_clause()?)?,
        ]))
    }

    fn try_catch_action(&mut self, action: &TryCatchAction) -> Option<Doc> {
        Some(concat([
            self.keyword(action.try_clause()?.delims()?, "try")?,
            self.body(action.try_body()?),
            self.keyword(action.catch_clause()?.delims()?, "catch")?,
            self.body(action.catch_body()?),
            self.end_clause(&action.end_clause()?)?,
        ]))
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
