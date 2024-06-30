mod assoc_templates;
mod conditionals;
mod expr;
mod loops;
mod try_catch;

use yag_template_syntax::ast;

use super::context::TypeckContext;
use super::flow::{Block, BlockKind};
use super::output::TypeckOutput;
use super::ty::Ty;

pub(crate) fn check(root: ast::Root) -> TypeckOutput {
    todo!()
}

pub(crate) fn check_template_body(ctx: &mut TypeckContext, actions: impl Iterator<Item = ast::Action>) {}

pub(crate) fn check_action_list(
    ctx: &mut TypeckContext,
    kind: BlockKind,
    context_ty: Ty,
    list: Option<ast::ActionList>,
) -> Block {
    let Some(list) = list else {
        return Block::never();
    };
    todo!()
}

pub(crate) fn check_action(ctx: &mut TypeckContext, action: ast::Action) {
    todo!()
}
