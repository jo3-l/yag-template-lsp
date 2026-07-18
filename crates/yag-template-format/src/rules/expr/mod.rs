//! Declarative document rules for expressions.

use yag_template_syntax::ast::{AstNode, AstToken, Expr, ExprFieldChain, Pipeline};

use crate::lower::Formatter;
use crate::pretty::{Doc, GroupId, concat, empty, group, group_with_id, if_break, line, soft_line, text, try_concat};

mod call;

/// A lowered expression with metadata about its trailing closing boundary.
pub(crate) struct ExprDoc {
    pub(crate) doc: Doc,
    /// The named group which, when broken, generates the expression's final
    /// closing-delimiter row.
    pub(crate) trailing_closing_group: Option<GroupId>,
}

impl ExprDoc {
    pub(crate) fn new(doc: Doc) -> Self {
        Self {
            doc,
            trailing_closing_group: None,
        }
    }

    pub(crate) fn with_prefix(self, prefix: Doc) -> Self {
        Self {
            doc: concat([prefix, self.doc]),
            // Prefixing does not change where an expression ends.
            trailing_closing_group: self.trailing_closing_group,
        }
    }

    pub(crate) fn with_suffix(self, suffix: Doc) -> Self {
        // Appending syntax means the expression no longer ends at an earlier closing row.
        Self::new(concat([self.doc, suffix]))
    }

    pub(crate) fn into_doc(self) -> Doc {
        self.doc
    }
}

impl Formatter<'_> {
    /// Format an explicit expression as a document fragment.
    pub(crate) fn expr(&mut self, expr: Expr) -> Option<ExprDoc> {
        self.expr_doc(expr)
    }

    /// Lower an expression following a keyword, assignment operator, or other
    /// prefix. Calls and opening parentheses stay with that prefix and own
    /// their internal breaks.
    pub(crate) fn prefixed_expr(&mut self, expr: Expr) -> Option<ExprDoc> {
        self.prefixed_expr_doc(expr)
    }
}

impl Formatter<'_> {
    fn expr_doc(&mut self, expr: Expr) -> Option<ExprDoc> {
        match expr {
            Expr::FuncCall(expr) => self.func_call(expr),
            Expr::ExprCall(expr) => self.expr_call(expr),
            Expr::Parenthesized(expr) => {
                let inner = self.expr_doc(expr.inner_expr()?)?;
                let id = self.new_group_id();
                Some(ExprDoc {
                    doc: group_with_id(
                        id,
                        concat([text("("), inner.into_doc(), if_break(id, line(), empty()), text(")")]),
                    ),
                    trailing_closing_group: Some(id),
                })
            }
            Expr::Pipeline(expr) => self.pipeline(expr),
            Expr::ContextAccess(expr) => Some(ExprDoc::new(text(expr.syntax().text().to_owned()))),
            Expr::ContextFieldChain(expr) => Some(ExprDoc::new(concat(
                expr.fields().map(|field| text(field.syntax().text().to_owned())),
            ))),
            Expr::ExprFieldChain(expr) => self.expr_field_chain(expr),
            Expr::VarAccess(expr) => Some(ExprDoc::new(text(expr.var()?.name()))),
            Expr::VarDecl(expr) => {
                self.assignment(":=", expr.var().map(|var| var.name().to_owned()), expr.initializer())
            }
            Expr::VarAssign(expr) => {
                self.assignment("=", expr.var().map(|var| var.name().to_owned()), expr.assign_expr())
            }
            Expr::Literal(expr) => Some(ExprDoc::new(text(expr.syntax().text().to_owned()))),
        }
    }

    fn pipeline(&mut self, expr: Pipeline) -> Option<ExprDoc> {
        let initial = self.expr_doc(expr.init_expr()?)?;
        let stages = expr
            .stages()
            .map(|stage| self.expr_doc(stage.call_expr()?))
            .collect::<Option<Vec<_>>>()?;
        let trailing_closing_group = stages
            .last()
            .map_or(initial.trailing_closing_group, |stage| stage.trailing_closing_group);
        let stages = try_concat(
            stages
                .into_iter()
                .map(|stage| Some(concat([soft_line(), text("| "), stage.into_doc()]))),
        )?;
        Some(ExprDoc {
            doc: group(concat([initial.into_doc(), self.indent_if_broken(stages)])),
            trailing_closing_group,
        })
    }

    fn expr_field_chain(&mut self, expr: ExprFieldChain) -> Option<ExprDoc> {
        Some(self.expr_doc(expr.base_expr()?)?.with_suffix(concat(
            expr.fields().map(|field| text(field.syntax().text().to_owned())),
        )))
    }

    fn prefixed_expr_doc(&mut self, expr: Expr) -> Option<ExprDoc> {
        // Calls and parentheses own their first possible break, so their opening
        // head stays beside the prefix. Other expressions may break before the
        // entire expression.
        let stays_with_prefix = matches!(expr, Expr::FuncCall(_) | Expr::ExprCall(_) | Expr::Parenthesized(_));
        let expr = self.expr_doc(expr)?;
        if stays_with_prefix {
            Some(expr.with_prefix(text(" ")))
        } else {
            let trailing_closing_group = expr.trailing_closing_group;
            Some(ExprDoc {
                doc: self.indent_if_broken(concat([soft_line(), expr.into_doc()])),
                trailing_closing_group,
            })
        }
    }

    fn assignment(&mut self, operator: &str, variable: Option<String>, value: Option<Expr>) -> Option<ExprDoc> {
        let value = self.prefixed_expr_doc(value?)?;
        let trailing_closing_group = value.trailing_closing_group;
        Some(ExprDoc {
            doc: group(concat([text(variable?), text(" "), text(operator), value.into_doc()])),
            trailing_closing_group,
        })
    }
}
