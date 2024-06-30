use yag_template_syntax::ast::Expr;

use crate::typeck::context::TypeckContext;
use crate::typeck::ty::Ty;

pub(crate) fn check_expr(ctx: &mut TypeckContext, expr: Option<Expr>) -> Ty {
    let Some(expr) = expr else {
        return Ty::Any;
    };
    todo!()
}
