//! Declarative document rules for expressions.

use yag_template_syntax::ast::{AstNode, AstToken, Expr, ExprCall, ExprFieldChain, FuncCall, Pipeline};

use crate::LayoutKind;
use crate::lower::Formatter;
use crate::pretty::{Doc, concat, group, soft_line, text, try_concat};

impl Formatter<'_> {
    /// Format an explicit expression variant as a document fragment.
    pub(crate) fn expr(&mut self, expr: Expr) -> Option<Doc> {
        match expr {
            Expr::FuncCall(expr) => self.func_call(expr),
            Expr::ExprCall(expr) => self.expr_call(expr),
            Expr::Parenthesized(expr) => Some(concat([text("("), self.expr(expr.inner_expr()?)?, text(")")])),
            Expr::Pipeline(expr) => self.pipeline(expr),
            Expr::ContextAccess(expr) => Some(text(expr.syntax().text().to_owned())),
            Expr::ContextFieldChain(expr) => Some(concat(
                expr.fields().map(|field| text(field.syntax().text().to_owned())),
            )),
            Expr::ExprFieldChain(expr) => self.expr_field_chain(expr),
            Expr::VarAccess(expr) => Some(text(expr.var()?.name())),
            Expr::VarDecl(expr) => {
                self.assignment(":=", expr.var().map(|var| var.name().to_owned()), expr.initializer())
            }
            Expr::VarAssign(expr) => {
                self.assignment("=", expr.var().map(|var| var.name().to_owned()), expr.assign_expr())
            }
            Expr::Literal(expr) => Some(text(expr.syntax().text().to_owned())),
        }
    }

    fn func_call(&mut self, expr: FuncCall) -> Option<Doc> {
        let name = expr.func_name()?;
        let args = expr.args().map(|arg| self.expr(arg)).collect::<Option<Vec<_>>>()?;
        Some(self.function_call(name.get(), args))
    }

    fn expr_call(&mut self, expr: ExprCall) -> Option<Doc> {
        let callee = self.expr(expr.callee()?)?;
        let args = expr.args().map(|arg| self.expr(arg)).collect::<Option<Vec<_>>>()?;
        Some(self.call(callee, args))
    }

    fn pipeline(&mut self, expr: Pipeline) -> Option<Doc> {
        let initial = self.expr(expr.init_expr()?)?;
        let stages = try_concat(
            expr.stages()
                .map(|stage| Some(concat([soft_line(), text("| "), self.expr(stage.call_expr()?)?]))),
        )?;
        Some(group(concat([initial, self.continuation(stages)])))
    }

    fn expr_field_chain(&mut self, expr: ExprFieldChain) -> Option<Doc> {
        Some(concat([
            self.expr(expr.base_expr()?)?,
            concat(expr.fields().map(|field| text(field.syntax().text().to_owned()))),
        ]))
    }

    fn function_call(&mut self, name: &str, args: Vec<Doc>) -> Doc {
        match self.function_layout(name) {
            Some(LayoutKind::KeyValuePairs) if args.len().is_multiple_of(2) => self.key_value_call(text(name), args),
            Some(LayoutKind::KeyValuePairs) => self.call(text(name), args),
            Some(LayoutKind::Call) | None => self.call(text(name), args),
        }
    }

    fn call(&self, callee: Doc, args: impl IntoIterator<Item = Doc>) -> Doc {
        let args = args.into_iter().collect::<Vec<_>>();
        if args.is_empty() {
            return callee;
        }
        group(concat([
            callee,
            self.continuation(concat(args.into_iter().flat_map(|arg| [soft_line(), arg]))),
        ]))
    }

    fn key_value_call(&self, callee: Doc, args: Vec<Doc>) -> Doc {
        let rows = args
            .chunks_exact(2)
            .flat_map(|pair| [soft_line(), pair[0].clone(), text(" "), pair[1].clone()]);
        group(concat([callee, self.continuation(concat(rows))]))
    }

    fn assignment(&mut self, operator: &str, variable: Option<String>, value: Option<Expr>) -> Option<Doc> {
        let value = self.expr(value?)?;
        Some(group(concat([
            text(variable?),
            text(" "),
            text(operator),
            self.continuation(concat([soft_line(), value])),
        ])))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{FormatOptions, FunctionLayouts, LayoutKind, format};

    #[test]
    fn configured_key_value_functions_dispatch_by_exact_name() {
        let options = FormatOptions {
            max_width: 18,
            function_layouts: FunctionLayouts {
                by_name: HashMap::from([("metadata".to_owned(), LayoutKind::KeyValuePairs)]),
            },
            ..FormatOptions::default()
        };
        let source = "{{metadata \"name\" (print .First .Last) \"active\" true}}";
        let expected = "{{ metadata\n\t\"name\" (print\n\t\t.First\n\t\t.Last)\n\t\"active\" true\n}}";

        assert_eq!(format(source, &options).text, expected);
    }
}
