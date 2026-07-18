//! Declarative document rules for expressions.

use yag_template_envdefs::EnvDefs;
use yag_template_syntax::ast::{AstNode, AstToken, Expr, ExprCall, ExprFieldChain, FuncCall, Pipeline};

use crate::lower::Formatter;
use crate::pretty::{Doc, GroupId, concat, empty, group, group_with_id, if_break, line, soft_line, text, try_concat};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum CallLayout {
    Default,
    Variadic { fixed_count: usize, rows: VariadicRows },
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum VariadicRows {
    Arguments,
    KeyValuePairs,
}

fn classify_call(envdefs: &EnvDefs, name: &str, actual_count: usize) -> CallLayout {
    let Some(function) = envdefs.funcs.get(name) else {
        return CallLayout::Default;
    };
    let Some(variadic) = function.params.last().filter(|param| param.is_variadic) else {
        return CallLayout::Default;
    };

    let fixed_count = function.params.len() - 1;
    if actual_count <= fixed_count {
        return CallLayout::Default;
    }

    let rows = if matches!(variadic.name.as_str(), "opts" | "keyvalues") {
        if !(actual_count - fixed_count).is_multiple_of(2) {
            return CallLayout::Default;
        }
        VariadicRows::KeyValuePairs
    } else {
        VariadicRows::Arguments
    };
    CallLayout::Variadic { fixed_count, rows }
}

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

    fn with_suffix(self, suffix: Doc) -> Self {
        // Appending syntax means the expression no longer ends at an earlier closing row.
        Self::new(concat([self.doc, suffix]))
    }

    fn into_doc(self) -> Doc {
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
}

impl<'a> Formatter<'a> {
    fn func_call(&mut self, expr: FuncCall) -> Option<ExprDoc> {
        let name = expr.func_name()?;
        let args = expr.args().map(|arg| self.expr_doc(arg)).collect::<Option<Vec<_>>>()?;
        Some(self.function_call(name.get(), args))
    }

    fn expr_call(&mut self, expr: ExprCall) -> Option<ExprDoc> {
        let callee = self.expr_doc(expr.callee()?)?.into_doc();
        let args = expr.args().map(|arg| self.expr_doc(arg)).collect::<Option<Vec<_>>>()?;
        Some(self.call(callee, args))
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

    fn function_call(&mut self, name: &str, args: Vec<ExprDoc>) -> ExprDoc {
        // Slice 5 will construct all signature-guided layouts. For now, retain
        // the existing output gate while deriving that decision from the full classifier.
        match classify_call(self.envdefs, name, args.len()) {
            CallLayout::Variadic {
                fixed_count: 0,
                rows: VariadicRows::KeyValuePairs,
            } => self.key_value_call(text(name), args),
            CallLayout::Default | CallLayout::Variadic { .. } => self.call(text(name), args),
        }
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

    fn call(&self, callee: Doc, args: impl IntoIterator<Item = ExprDoc>) -> ExprDoc {
        let args = args.into_iter().collect::<Vec<_>>();
        if args.is_empty() {
            return ExprDoc::new(callee);
        }
        let trailing_closing_group = args.last().and_then(|arg| arg.trailing_closing_group);
        ExprDoc {
            doc: group(concat([
                callee,
                self.indent_if_broken(concat(args.into_iter().flat_map(|arg| [soft_line(), arg.into_doc()]))),
            ])),
            trailing_closing_group,
        }
    }

    fn key_value_call(&self, callee: Doc, args: Vec<ExprDoc>) -> ExprDoc {
        let trailing_closing_group = args.last().and_then(|arg| arg.trailing_closing_group);
        let args = args.into_iter().map(ExprDoc::into_doc).collect::<Vec<_>>();
        let rows = args
            .chunks_exact(2)
            .flat_map(|pair| [soft_line(), pair[0].clone(), text(" "), pair[1].clone()]);
        ExprDoc {
            doc: group(concat([callee, self.indent_if_broken(concat(rows))])),
            trailing_closing_group,
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

#[cfg(test)]
mod tests {
    use yag_template_envdefs::EnvDefSource;

    use super::{CallLayout, VariadicRows, classify_call};

    fn envdefs() -> yag_template_envdefs::EnvDefs {
        yag_template_envdefs::parse(&[EnvDefSource::new_static(
            "test.ydef",
            concat!(
                "func fixed(value)\n",
                "func values(args...)\n",
                "func parseArgs(first, description, argDefs...)\n",
                "func mixed(first, opts...)\n",
                "func keys(keyvalues...)\n",
                "func Set(opts...)\n",
            ),
        )])
        .unwrap()
    }

    #[test]
    fn call_classification_uses_variadic_signature_shape() {
        let envdefs = envdefs();
        assert_eq!(classify_call(&envdefs, "unknown", 3), CallLayout::Default);
        assert_eq!(classify_call(&envdefs, "fixed", 1), CallLayout::Default);
        assert_eq!(
            classify_call(&envdefs, "values", 3),
            CallLayout::Variadic {
                fixed_count: 0,
                rows: VariadicRows::Arguments,
            }
        );
        assert_eq!(
            classify_call(&envdefs, "parseArgs", 4),
            CallLayout::Variadic {
                fixed_count: 2,
                rows: VariadicRows::Arguments,
            }
        );
        assert_eq!(
            classify_call(&envdefs, "mixed", 5),
            CallLayout::Variadic {
                fixed_count: 1,
                rows: VariadicRows::KeyValuePairs,
            }
        );
    }

    #[test]
    fn expr_callees_do_not_use_function_signature_layouts() {
        let options = crate::FormatOptions {
            max_width: 16,
            ..crate::FormatOptions::default()
        };
        let result = crate::format(r#"{{.Set "a" "one" "b" "two"}}"#, &envdefs(), &options);
        assert!(result.diagnostics.is_empty());
        assert!(result.text.contains("\t\"a\"\n\t\"one\""), "{}", result.text);
    }

    #[test]
    fn key_value_and_short_variadic_calls_fall_back_when_structure_is_unavailable() {
        let envdefs = envdefs();
        assert_eq!(classify_call(&envdefs, "mixed", 4), CallLayout::Default);
        assert_eq!(classify_call(&envdefs, "parseArgs", 2), CallLayout::Default);
        assert_eq!(classify_call(&envdefs, "parseArgs", 1), CallLayout::Default);
        assert_eq!(classify_call(&envdefs, "keys", 3), CallLayout::Default);
        assert_eq!(
            classify_call(&envdefs, "keys", 4),
            CallLayout::Variadic {
                fixed_count: 0,
                rows: VariadicRows::KeyValuePairs,
            }
        );
    }
}
