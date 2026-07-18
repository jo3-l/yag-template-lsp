//! Declarative document rules for template actions and clauses.

use yag_template_syntax::ast::{
    Action, AstNode, AstToken, ElseBranch, ElseClause, EndClause, Expr, IfAction, LeftDelim, RangeClause, RangeLoop,
    RightDelim, TemplateBlock, TemplateDefinition, TryCatchAction, WhileLoop, WithAction,
};

use crate::DelimiterPadding;
use crate::lower::Formatter;
use crate::pretty::{AllowCompact, Doc, concat, empty, group_with_id, if_break, join, line, text, try_concat};
use crate::rules::expr::LoweredExpr;

impl Formatter<'_> {
    /// Format an action atomically. A rule that cannot construct a complete
    /// document leaves the original typed action untouched.
    pub(crate) fn action(&mut self, action: Action) -> Doc {
        self.action_rule(&action)
            .unwrap_or_else(|| text(action.syntax().text().to_owned()))
    }

    fn delimited_doc(&mut self, delims: (LeftDelim, RightDelim), body: Doc) -> Doc {
        self.delimited(delims, LoweredExpr::new(body))
    }

    fn delimited(&mut self, (left_delim, right_delim): (LeftDelim, RightDelim), body: LoweredExpr) -> Doc {
        let LoweredExpr {
            doc: body,
            trailing_closing_group,
        } = body;
        let pad_delimiters = self.options.delimiter_padding == DelimiterPadding::Spaces;

        // Opening padding is horizontal: trim delimiters require one space,
        // while ordinary delimiters follow the configured padding.
        let left = if left_delim.has_trim_marker() {
            text("{{- ")
        } else if pad_delimiters {
            text("{{ ")
        } else {
            text("{{")
        };

        // Closing padding is emitted only when the delimiter stays on the same
        // row. A generated newline replaces it, so no line ends in padding.
        let closing_padding = if right_delim.has_trim_marker() || pad_delimiters {
            text(" ")
        } else {
            empty()
        };
        let right = if right_delim.has_trim_marker() {
            text("-}}")
        } else {
            text("}}")
        };

        let action_id = self.new_group_id();
        let action_boundary = if_break(action_id, line(), closing_padding.clone());
        let closing_boundary = trailing_closing_group.map_or(action_boundary.clone(), |closing_id| {
            // Reuse a trailing parenthesis row when one exists; otherwise the
            // action group decides whether the right delimiter needs a new row.
            if_break(closing_id, closing_padding, action_boundary)
        });
        group_with_id(action_id, concat([left, body, closing_boundary, right]))
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
            .group_if(allow_compact),
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
            .group_if(allow_compact),
        )
    }

    fn if_action(&mut self, action: &IfAction) -> Option<Doc> {
        let allow_compact = allows_compact(action);
        let clause = action.clause()?;
        Some(
            concat([
                self.kw_then_expr(clause.delims()?, "if", clause.condition()?)?,
                self.body(action.body()?, allow_compact),
                self.else_branches(action.else_branches(), allow_compact)?,
                self.end_clause(&action.end_clause()?)?,
            ])
            .group_if(allow_compact),
        )
    }

    fn with_action(&mut self, action: &WithAction) -> Option<Doc> {
        let allow_compact = allows_compact(action);
        let clause = action.clause()?;
        Some(
            concat([
                self.kw_then_expr(clause.delims()?, "with", clause.condition()?)?,
                self.body(action.body()?, allow_compact),
                self.else_branches(action.else_branches(), allow_compact)?,
                self.end_clause(&action.end_clause()?)?,
            ])
            .group_if(allow_compact),
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
        match clause.condition() {
            Some(condition) => self.kw_then_expr(clause.delims()?, "else if", condition),
            None => self.kw(clause.delims()?, "else"),
        }
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
            .group_if(allow_compact),
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
        let expr = self.prefixed_expr(clause.expr()?)?;
        Some(self.delimited(clause.delims()?, expr.with_prefix(concat([text("range"), binding]))))
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
                self.kw_then_expr(clause.delims()?, "while", clause.condition()?)?,
                self.body(action.body()?, allow_compact),
                else_branch,
                self.end_clause(&action.end_clause()?)?,
            ])
            .group_if(allow_compact),
        )
    }

    fn try_catch_action(&mut self, action: &TryCatchAction) -> Option<Doc> {
        let allow_compact = allows_compact(action);
        Some(
            concat([
                self.kw(action.try_clause()?.delims()?, "try")?,
                self.body(action.try_body()?, allow_compact),
                self.kw(action.catch_clause()?.delims()?, "catch")?,
                self.body(action.catch_body()?, allow_compact),
                self.end_clause(&action.end_clause()?)?,
            ])
            .group_if(allow_compact),
        )
    }

    fn end_clause(&mut self, clause: &EndClause) -> Option<Doc> {
        self.kw(clause.delims()?, "end")
    }

    fn template_action(
        &mut self,
        delims: (LeftDelim, RightDelim),
        kw: &str,
        template_name: String,
        context_data: Option<Expr>,
    ) -> Option<Doc> {
        let prefix = concat([text(kw), text(" "), text(template_name)]);
        match context_data {
            Some(context_data) => {
                let context_data = self.prefixed_expr(context_data)?.with_prefix(prefix);
                Some(self.delimited(delims, context_data))
            }
            None => Some(self.delimited_doc(delims, prefix)),
        }
    }

    fn kw(&mut self, delims: (LeftDelim, RightDelim), kw: &str) -> Option<Doc> {
        Some(self.delimited_doc(delims, text(kw)))
    }

    fn kw_then_expr(&mut self, delims: (LeftDelim, RightDelim), kw: &str, expr: Expr) -> Option<Doc> {
        let expr = self.prefixed_expr(expr)?.with_prefix(text(kw));
        Some(self.delimited(delims, expr))
    }
}

fn allows_compact(action: &impl AstNode) -> AllowCompact {
    if Action::cast(action.syntax().clone()).is_some_and(is_compact_action) {
        AllowCompact::Yes
    } else {
        AllowCompact::No
    }
}

/// Whether a typed action can share a compact compound layout with its
/// enclosing clause. This is based only on the action tree: formatter-owned
/// whitespace introduced while reflowing an expression cannot change it on a
/// later pass.
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
fn is_compact_body(body: yag_template_syntax::ast::ActionList) -> bool {
    let mut action = None;
    for element in body.actions_with_text() {
        match element {
            yag_template_syntax::ast::ActionOrText::Action(next) => {
                if action.replace(next).is_some() {
                    return false;
                }
            }
            yag_template_syntax::ast::ActionOrText::Text(text) if text.get().chars().all(char::is_whitespace) => {}
            yag_template_syntax::ast::ActionOrText::Text(_) => return false,
        }
    }
    action.is_none_or(is_compact_action)
}

#[cfg(test)]
mod tests {
    use yag_template_syntax::SyntaxNode;
    use yag_template_syntax::ast::{AstNode, Root};

    use super::{AllowCompact, allows_compact};

    fn compactness(source: &str) -> AllowCompact {
        let parsed = yag_template_syntax::parser::parse(source);
        assert!(parsed.errors.is_empty(), "source did not parse: {:?}", parsed.errors);
        let root = Root::cast(SyntaxNode::new_root(parsed.root)).unwrap();
        allows_compact(&root.actions().next().unwrap())
    }

    #[test]
    fn compound_compactness_follows_direct_body_actions_recursively() {
        assert_eq!(
            compactness("{{if .Enabled}} {{ $name := .Name }} {{end}}"),
            AllowCompact::Yes
        );
        assert_eq!(
            compactness("{{if .Enabled}} {{ $name := .Name }} {{ $value := .Value }} {{end}}"),
            AllowCompact::No
        );
        assert_eq!(
            compactness("{{if .Enabled}} {{ $name := .Name }} {{else}} {{ $value := .Value }} {{end}}"),
            AllowCompact::Yes
        );
        assert_eq!(
            compactness("{{range .Items}}{{if .Enabled}}{{ $name := .Name}}{{ $value := .Value}}{{end}}{{end}}"),
            AllowCompact::No
        );
    }
}
