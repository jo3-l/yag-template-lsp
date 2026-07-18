//! Declarative document rules for template actions.

use yag_template_syntax::ast::{
    Action, ActionList, ActionOrText, AstNode, AstToken, ElseBranch, ElseClause, EndClause, Expr, IfAction, LeftDelim,
    RangeClause, RangeLoop, RightDelim, TemplateBlock, TemplateDefinition, TryCatchAction, WhileLoop, WithAction,
};

use crate::lower::Formatter;
use crate::pretty::{Doc, concat, empty, join, space, text, try_concat};
use crate::rules::delimited::DelimitedInner;

impl Formatter<'_> {
    /// Format an action atomically. A rule that cannot construct a complete
    /// document leaves the original typed action untouched.
    pub(super) fn action(&mut self, action: Action) -> Doc {
        let mut try_format = || -> Option<Doc> {
            match &action {
                Action::Comment(action) => Some(text(action.syntax().text().to_owned())),
                Action::TemplateDefinition(action) => self.template_definition(action),
                Action::TemplateBlock(action) => self.template_block(action),
                Action::TemplateInvocation(action) => self.template_or_block_clause(
                    action.delims()?,
                    "template",
                    action.template_name()?.syntax().text().to_string(),
                    action.context_data(),
                ),
                Action::Return(action) => self.kw_then_expr(action.delims()?, "return", action.expr()?),
                Action::If(action) => self.if_action(action),
                Action::With(action) => self.with_action(action),
                Action::Range(action) => self.range_action(action),
                Action::While(action) => self.while_action(action),
                Action::Break(action) => self.kw(action.delims()?, "break"),
                Action::Continue(action) => self.kw(action.delims()?, "continue"),
                Action::TryCatch(action) => self.try_catch_action(action),
                Action::ExprAction(action) => {
                    let expr = self.expr(action.expr()?)?;
                    Some(self.delimited(action.delims()?, expr))
                }
            }
        };
        try_format().unwrap_or_else(|| text(action.syntax().text().to_owned()))
    }

    fn template_definition(&mut self, tmpldef: &TemplateDefinition) -> Option<Doc> {
        let allow_compact = allows_compact(tmpldef);

        let clause = tmpldef.clause()?;
        let formatted_clause = self.delimited(
            clause.delims()?,
            DelimitedInner::new(concat([
                text("define "),
                text(clause.template_name()?.syntax().text().to_string()),
            ])),
        );
        Some(
            concat([
                formatted_clause,
                self.body(tmpldef.template_body()?, allow_compact),
                self.end_clause(&tmpldef.end_clause()?)?,
            ])
            .group_if(allow_compact),
        )
    }

    fn template_block(&mut self, tmplblock: &TemplateBlock) -> Option<Doc> {
        let allow_compact = allows_compact(tmplblock);
        let clause = tmplblock.clause()?;
        Some(
            concat([
                self.template_or_block_clause(
                    clause.delims()?,
                    "block",
                    clause.template_name()?.syntax().text().to_string(),
                    clause.context_data(),
                )?,
                self.body(tmplblock.template_body()?, allow_compact),
                self.end_clause(&tmplblock.end_clause()?)?,
            ])
            .group_if(allow_compact),
        )
    }

    fn if_action(&mut self, if_: &IfAction) -> Option<Doc> {
        let allow_compact = allows_compact(if_);
        let clause = if_.clause()?;
        Some(
            concat([
                self.kw_then_expr(clause.delims()?, "if", clause.condition()?)?,
                self.body(if_.body()?, allow_compact),
                self.else_branches(if_.else_branches(), allow_compact)?,
                self.end_clause(&if_.end_clause()?)?,
            ])
            .group_if(allow_compact),
        )
    }

    fn with_action(&mut self, with: &WithAction) -> Option<Doc> {
        let allow_compact = allows_compact(with);
        let clause = with.clause()?;
        Some(
            concat([
                self.kw_then_expr(clause.delims()?, "with", clause.condition()?)?,
                self.body(with.body()?, allow_compact),
                self.else_branches(with.else_branches(), allow_compact)?,
                self.end_clause(&with.end_clause()?)?,
            ])
            .group_if(allow_compact),
        )
    }

    fn else_branches(&mut self, branches: impl Iterator<Item = ElseBranch>, allow_compact: bool) -> Option<Doc> {
        try_concat(branches.map(|branch| {
            let clause = branch.clause()?;
            Some(concat([
                self.else_clause(&clause)?,
                self.body(branch.body()?, allow_compact),
            ]))
        }))
    }

    fn else_clause(&mut self, end: &ElseClause) -> Option<Doc> {
        match end.condition() {
            Some(condition) => self.kw_then_expr(end.delims()?, "else if", condition),
            None => self.kw(end.delims()?, "else"),
        }
    }

    fn range_action(&mut self, range: &RangeLoop) -> Option<Doc> {
        let allow_compact = allows_compact(range);
        let clause = range.clause()?;
        let else_branch = if let Some(branch) = range.else_branch() {
            let clause = branch.clause()?;
            concat([self.else_clause(&clause)?, self.body(branch.body()?, allow_compact)])
        } else {
            empty()
        };
        Some(
            concat([
                self.range_clause(&clause)?,
                self.body(range.body()?, allow_compact),
                else_branch,
                self.end_clause(&range.end_clause()?)?,
            ])
            .group_if(allow_compact),
        )
    }

    fn range_clause(&mut self, range_clause: &RangeClause) -> Option<Doc> {
        let mut variables = range_clause.iteration_vars().peekable();
        let binding = if variables.peek().is_some() {
            let assignment = if range_clause.declares_vars() {
                text(" :=")
            } else if range_clause.assigns_vars() {
                text(" =")
            } else {
                empty()
            };
            concat([
                space(),
                join(text(", "), variables.map(|variable| text(variable.name()))),
                assignment,
            ])
        } else {
            empty()
        };
        let expr = self.prefixed_expr(range_clause.expr()?)?;
        Some(self.delimited(
            range_clause.delims()?,
            expr.with_prefix(concat([text("range"), binding])),
        ))
    }

    fn while_action(&mut self, while_: &WhileLoop) -> Option<Doc> {
        let allow_compact = allows_compact(while_);
        let clause = while_.clause()?;
        let else_branch = if let Some(branch) = while_.else_branch() {
            let clause = branch.clause()?;
            concat([self.else_clause(&clause)?, self.body(branch.body()?, allow_compact)])
        } else {
            empty()
        };
        Some(
            concat([
                self.kw_then_expr(clause.delims()?, "while", clause.condition()?)?,
                self.body(while_.body()?, allow_compact),
                else_branch,
                self.end_clause(&while_.end_clause()?)?,
            ])
            .group_if(allow_compact),
        )
    }

    fn try_catch_action(&mut self, tc: &TryCatchAction) -> Option<Doc> {
        let allow_compact = allows_compact(tc);
        Some(
            concat([
                self.kw(tc.try_clause()?.delims()?, "try")?,
                self.body(tc.try_body()?, allow_compact),
                self.kw(tc.catch_clause()?.delims()?, "catch")?,
                self.body(tc.catch_body()?, allow_compact),
                self.end_clause(&tc.end_clause()?)?,
            ])
            .group_if(allow_compact),
        )
    }

    fn end_clause(&mut self, end: &EndClause) -> Option<Doc> {
        self.kw(end.delims()?, "end")
    }

    fn template_or_block_clause(
        &mut self,
        delims: (LeftDelim, RightDelim),
        keyword: &str,
        template_name: String,
        context_data: Option<Expr>,
    ) -> Option<Doc> {
        let prefix = concat([text(keyword), space(), text(template_name)]);
        let inner = match context_data {
            Some(context_data) => self.prefixed_expr(context_data)?.with_prefix(prefix),
            None => DelimitedInner::new(prefix),
        };
        Some(self.delimited(delims, inner))
    }

    fn kw(&mut self, delims: (LeftDelim, RightDelim), kw: &str) -> Option<Doc> {
        Some(self.delimited(delims, DelimitedInner::new(text(kw))))
    }

    fn kw_then_expr(&mut self, delims: (LeftDelim, RightDelim), kw: &str, expr: Expr) -> Option<Doc> {
        let expr = self.prefixed_expr(expr)?.with_prefix(text(kw));
        Some(self.delimited(delims, expr))
    }
}

fn allows_compact(action: &impl AstNode) -> bool {
    Action::cast(action.syntax().clone()).is_some_and(is_compact_action)
}

/// Whether a typed action can share a compact compound layout with its
/// enclosing clause.
fn is_compact_action(action: Action) -> bool {
    match action {
        Action::Comment(action) => !action.syntax().text().contains_char('\n'),
        Action::TemplateDefinition(action) => action.template_body().is_some_and(is_compact_body),
        Action::TemplateBlock(action) => action.template_body().is_some_and(is_compact_body),
        Action::If(action) => {
            action.body().is_some_and(is_compact_body)
                && action
                    .else_branches()
                    .all(|branch| branch.body().is_some_and(is_compact_body))
        }
        Action::With(action) => {
            action.body().is_some_and(is_compact_body)
                && action
                    .else_branches()
                    .all(|branch| branch.body().is_some_and(is_compact_body))
        }
        Action::Range(action) => {
            action.body().is_some_and(is_compact_body)
                && action
                    .else_branch()
                    .is_none_or(|branch| branch.body().is_some_and(is_compact_body))
        }
        Action::While(action) => {
            action.body().is_some_and(is_compact_body)
                && action
                    .else_branch()
                    .is_none_or(|branch| branch.body().is_some_and(is_compact_body))
        }
        Action::TryCatch(action) => {
            action.try_body().is_some_and(is_compact_body) && action.catch_body().is_some_and(is_compact_body)
        }
        Action::TemplateInvocation(_)
        | Action::Return(_)
        | Action::Break(_)
        | Action::Continue(_)
        | Action::ExprAction(_) => true,
    }
}

/// A body is compact when it contains at most one compact action and no
/// literal content. Whitespace is deliberately ignored: it is either an
/// existing structural line or formatter-owned layout whitespace.
fn is_compact_body(body: ActionList) -> bool {
    let mut action = None;
    for element in body.actions_with_text() {
        match element {
            ActionOrText::Action(next) => {
                if action.replace(next).is_some() {
                    return false;
                }
            }
            ActionOrText::Text(text) if text.get().chars().all(char::is_whitespace) => {}
            ActionOrText::Text(_) => return false,
        }
    }
    action.is_none_or(is_compact_action)
}

#[cfg(test)]
mod tests {
    use yag_template_syntax::SyntaxNode;
    use yag_template_syntax::ast::{AstNode, Root};

    use super::allows_compact;

    fn compact(source: &str) -> bool {
        let parsed = yag_template_syntax::parser::parse(source);
        assert!(parsed.errors.is_empty(), "source did not parse: {:?}", parsed.errors);
        let root = Root::cast(SyntaxNode::new_root(parsed.root)).unwrap();
        allows_compact(&root.actions().next().unwrap())
    }

    #[test]
    fn compound_compactness_follows_direct_body_actions_recursively() {
        assert!(compact("{{if .Enabled}} {{ $name := .Name }} {{end}}"));
        assert!(!compact(
            "{{if .Enabled}} {{ $name := .Name }} {{ $value := .Value }} {{end}}"
        ));
        assert!(compact(
            "{{if .Enabled}} {{ $name := .Name }} {{else}} {{ $value := .Value }} {{end}}"
        ));
        assert!(!compact(
            "{{range .Items}}{{if .Enabled}}{{ $name := .Name}}{{ $value := .Value}}{{end}}{{end}}"
        ));
    }
}
