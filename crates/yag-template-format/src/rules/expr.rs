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
}

impl<'a> Formatter<'a> {
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
        Some(group(concat([initial, self.indent_if_broken(stages)])))
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

    fn is_direct_call(&self, expression: &Expr) -> bool {
        matches!(expression, Expr::FuncCall(_) | Expr::ExprCall(_))
    }

    /// Lower an expression following a keyword, assignment operator, or other
    /// prefix. Calls own their first break after the callee, so keep a direct
    /// call's callee with that prefix and let the call lay out its arguments.
    pub(crate) fn prefixed_expression(&mut self, expression: Expr) -> Option<Doc> {
        let is_call = self.is_direct_call(&expression);
        let expression = self.expr(expression)?;
        if is_call {
            Some(concat([text(" "), expression]))
        } else {
            Some(self.indent_if_broken(concat([soft_line(), expression])))
        }
    }

    fn call(&self, callee: Doc, args: impl IntoIterator<Item = Doc>) -> Doc {
        let args = args.into_iter().collect::<Vec<_>>();
        if args.is_empty() {
            return callee;
        }
        group(concat([
            callee,
            self.indent_if_broken(concat(args.into_iter().flat_map(|arg| [soft_line(), arg]))),
        ]))
    }

    fn key_value_call(&self, callee: Doc, args: Vec<Doc>) -> Doc {
        let rows = args
            .chunks_exact(2)
            .flat_map(|pair| [soft_line(), pair[0].clone(), text(" "), pair[1].clone()]);
        group(concat([callee, self.indent_if_broken(concat(rows))]))
    }

    fn assignment(&mut self, operator: &str, variable: Option<String>, value: Option<Expr>) -> Option<Doc> {
        let value = self.prefixed_expression(value?)?;
        Some(group(concat([text(variable?), text(" "), text(operator), value])))
    }
}
