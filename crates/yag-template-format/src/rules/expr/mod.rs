//! Declarative document rules for expressions.

use yag_template_syntax::ast::{AstNode, AstToken, Expr, ExprFieldChain, Pipeline};

use crate::lower::Formatter;
use crate::pretty::{concat, empty, group, group_with_id, if_break, line, soft_line, text, try_concat};
use crate::rules::delimited::DelimitedInner;

mod call;

impl Formatter<'_> {
    /// Format an explicit expression as a document fragment.
    pub(super) fn expr(&mut self, expr: Expr) -> Option<DelimitedInner> {
        match expr {
            Expr::FuncCall(expr) => self.func_call(expr),
            Expr::ExprCall(expr) => self.expr_call(expr),
            Expr::Parenthesized(expr) => {
                let inner = self.expr(expr.inner_expr()?)?;
                let id = self.new_group_id();
                Some(DelimitedInner {
                    doc: group_with_id(
                        id,
                        concat([text("("), inner.into_doc(), if_break(id, line(), empty()), text(")")]),
                    ),
                    trailing_closing_group: Some(id),
                })
            }
            Expr::Pipeline(expr) => self.pipeline(expr),
            Expr::ContextAccess(expr) => Some(DelimitedInner::new(text(expr.syntax().text().to_owned()))),
            Expr::ContextFieldChain(expr) => Some(DelimitedInner::new(concat(
                expr.fields().map(|field| text(field.syntax().text().to_owned())),
            ))),
            Expr::ExprFieldChain(expr) => self.expr_field_chain(expr),
            Expr::VarAccess(expr) => Some(DelimitedInner::new(text(expr.var()?.name()))),
            Expr::VarDecl(expr) => {
                self.assignment(":=", expr.var().map(|var| var.name().to_owned()), expr.initializer())
            }
            Expr::VarAssign(expr) => {
                self.assignment("=", expr.var().map(|var| var.name().to_owned()), expr.assign_expr())
            }
            Expr::Literal(expr) => Some(DelimitedInner::new(text(expr.syntax().text().to_owned()))),
        }
    }

    /// Lower an expression following a keyword, assignment operator, or other
    /// prefix. Calls and opening parentheses stay with that prefix and own
    /// their internal breaks.
    pub(super) fn prefixed_expr(&mut self, expr: Expr) -> Option<DelimitedInner> {
        // Calls and parentheses own their first possible break, so their opening
        // head stays beside the prefix. Other expressions may break before the
        // entire expression.
        let stays_with_prefix = matches!(expr, Expr::FuncCall(_) | Expr::ExprCall(_) | Expr::Parenthesized(_));
        let expr = self.expr(expr)?;
        if stays_with_prefix {
            Some(expr.with_prefix(text(" ")))
        } else {
            let trailing_closing_group = expr.trailing_closing_group;
            Some(DelimitedInner {
                doc: self.indent_if_broken(concat([soft_line(), expr.into_doc()])),
                trailing_closing_group,
            })
        }
    }
}

impl Formatter<'_> {
    fn pipeline(&mut self, pipe: Pipeline) -> Option<DelimitedInner> {
        let initial = self.expr(pipe.init_expr()?)?;
        let stages = pipe
            .stages()
            .map(|stage| self.expr(stage.call_expr()?))
            .collect::<Option<Vec<_>>>()?;
        let trailing_closing_group = stages
            .last()
            .map_or(initial.trailing_closing_group, |stage| stage.trailing_closing_group);
        let stages = try_concat(
            stages
                .into_iter()
                .map(|stage| Some(concat([soft_line(), text("| "), stage.into_doc()]))),
        )?;
        Some(DelimitedInner {
            doc: group(concat([initial.into_doc(), self.indent_if_broken(stages)])),
            trailing_closing_group,
        })
    }

    fn expr_field_chain(&mut self, chain: ExprFieldChain) -> Option<DelimitedInner> {
        Some(self.expr(chain.base_expr()?)?.with_suffix(concat(
            chain.fields().map(|field| text(field.syntax().text().to_owned())),
        )))
    }

    fn assignment(&mut self, op: &str, var: Option<String>, value: Option<Expr>) -> Option<DelimitedInner> {
        let value = self.prefixed_expr(value?)?;
        let trailing_closing_group = value.trailing_closing_group;
        Some(DelimitedInner {
            doc: group(concat([text(var?), text(" "), text(op), value.into_doc()])),
            trailing_closing_group,
        })
    }
}
