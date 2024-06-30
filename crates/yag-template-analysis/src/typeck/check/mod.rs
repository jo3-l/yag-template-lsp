mod assoc_templates;
mod conditionals;
mod expr;
mod loops;
mod try_catch;

use yag_template_syntax::ast;

use super::flow::{Block, BlockKind};
use super::ty::Ty;

pub(crate) fn check_action_list(kind: BlockKind, context_ty: Ty, list: Option<ast::ActionList>) -> Block {
    let Some(list) = list else {
        return Block::empty();
    };
    todo!()
}

pub(crate) fn check_action(action: ast::Action) {
    todo!()
}
