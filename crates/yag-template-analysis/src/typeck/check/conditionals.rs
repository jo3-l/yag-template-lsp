use yag_template_syntax::ast;

use super::check_action_list;
use super::expr::check_expr;
use crate::typeck::context::TypeckContext;
use crate::typeck::flow::{Block, BlockKind};

pub(crate) fn check_if_conditional(ctx: &mut TypeckContext, conditional: ast::IfConditional) -> Block {
    check_if_or_with(
        ctx,
        ConditionalKind::If,
        conditional.if_clause().and_then(|clause| clause.if_expr()),
        conditional.then_list(),
        conditional.else_branches(),
    )
}

pub(crate) fn check_with_conditional(ctx: &mut TypeckContext, conditional: ast::WithConditional) -> Block {
    check_if_or_with(
        ctx,
        ConditionalKind::With,
        conditional.with_clause().and_then(|clause| clause.with_expr()),
        conditional.then_list(),
        conditional.else_branches(),
    )
}

#[derive(Debug, PartialEq, Eq)]
enum ConditionalKind {
    If,
    With,
}

fn check_if_or_with(
    ctx: &mut TypeckContext,
    kind: ConditionalKind,
    cond: Option<ast::Expr>,
    then_list: Option<ast::ActionList>,
    mut else_branches: impl Iterator<Item = ast::ElseBranch>,
) -> Block {
    ctx.enter_block(BlockKind::default(), ctx.inherit_context_ty());

    let cond_ty = check_expr(ctx, cond);
    let context_ty = match kind {
        ConditionalKind::If => ctx.inherit_context_ty(),
        ConditionalKind::With => cond_ty,
    };
    let then_block = check_action_list(ctx, BlockKind::default(), context_ty, then_list);

    let else_block = match else_branches.next() {
        Some(branch) => {
            if branch.is_unconditional() {
                check_action_list(
                    ctx,
                    BlockKind::default(),
                    ctx.inherit_context_ty(),
                    branch.action_list(),
                )
            } else {
                // Process
                //   {{else if x}}
                //     ...
                //   {{else if y}}
                //     ...
                //   {{end}}
                // as
                //   {{else}}
                //     {{if x}}
                //       ...
                //     {{else if y}}
                //       ...
                //     {{end}}
                //   {{end}}
                let cond_expr = branch.else_clause().and_then(|clause| clause.cond_expr());
                check_if_or_with(ctx, ConditionalKind::If, cond_expr, branch.action_list(), else_branches)
            }
        }
        None => Block::never(),
    };

    let mut if_block = ctx.exit_block();
    if_block.merge_divergent_branches(then_block, else_block);
    if_block
}
