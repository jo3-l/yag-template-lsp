use yag_template_syntax::ast;

use super::check_action_list;
use crate::typeck::context::TypeckContext;
use crate::typeck::flow::{Block, BlockKind};

pub(crate) fn check_try_catch(ctx: &mut TypeckContext, try_catch: ast::TryCatchAction) -> Block {
    ctx.enter_block(BlockKind::default(), ctx.inherit_context_ty());

    let try_block = check_action_list(BlockKind::TryBody, ctx.inherit_context_ty(), try_catch.try_list());
    let catch_block = check_action_list(BlockKind::default(), try_block.throw_ty.clone(), try_catch.catch_list());

    let mut try_catch_block = ctx.exit_block();
    try_catch_block.merge_divergent_child_branches(try_block, catch_block);
    try_catch_block
}
