use yag_template_syntax::ast;

use crate::typeck::context::TypeckContext;
use crate::typeck::flow::Block;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ConditionalKind {
    If,
    With,
}

pub(crate) fn check_conditional(
    ctx: &TypeckContext,
    kind: ConditionalKind,
    cond_expr: ast::Expr,
    else_branches: impl Iterator<Item = ast::ElseBranch>,
) -> Block {
}
