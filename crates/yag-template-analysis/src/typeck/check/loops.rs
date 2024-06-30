use yag_template_syntax::ast;

use super::check_action_list;
use super::expr::check_expr;
use crate::typeck::context::TypeckContext;
use crate::typeck::flow::{Block, BlockKind};
use crate::typeck::ty::{base_ty, union_all, PrimitiveClass, PrimitiveTy, Ty};

pub(crate) fn check_while_loop(ctx: &mut TypeckContext, while_loop: ast::WhileLoop) -> Block {
    ctx.enter_block(BlockKind::default(), ctx.inherit_context_ty());

    check_expr(ctx, while_loop.while_clause().and_then(|clause| clause.cond_expr()));
    let while_body_block = check_action_list(BlockKind::LoopBody, ctx.inherit_context_ty(), while_loop.action_list());
    let while_else_block = check_action_list(
        BlockKind::default(),
        ctx.inherit_context_ty(),
        while_loop.else_branch().and_then(|branch| branch.action_list()),
    );

    let mut while_block = ctx.exit_block();
    while_block.merge_divergent_child_branches(while_body_block, while_else_block);
    while_block
}

pub(crate) fn check_range_loop(ctx: &mut TypeckContext, range: ast::RangeLoop) -> Block {
    ctx.enter_block(BlockKind::default(), ctx.inherit_context_ty());

    let range_header_and_body_block = check_range_header_and_body(ctx, range.range_clause(), range.action_list());
    let range_else_block = check_action_list(
        BlockKind::default(),
        ctx.inherit_context_ty(),
        range.else_branch().and_then(|branch| branch.action_list()),
    );

    let mut range_block = ctx.exit_block();
    range_block.merge_divergent_child_branches(range_header_and_body_block, range_else_block);
    range_block
}

fn check_range_header_and_body(
    ctx: &mut TypeckContext,
    range_clause: Option<ast::RangeClause>,
    range_list: Option<ast::ActionList>,
) -> Block {
    let Some(range_clause) = range_clause else {
        return Block::empty();
    };
    ctx.enter_block(BlockKind::default(), ctx.inherit_context_ty());

    let range_expr_ty = check_expr(ctx, range_clause.range_expr());
    let (key_ty, value_ty) = infer_iteration_var_types(ctx, &range_expr_ty);

    // Assign or declare the iteration variables.
    let mut iteration_vars = range_clause.iteration_vars();
    match (iteration_vars.next(), iteration_vars.next()) {
        (Some(first_var), Some(second_var)) => {
            if range_clause.assigns_vars() {
                ctx.assign(first_var.name(), key_ty);
                ctx.assign(second_var.name(), value_ty.clone());
            } else if range_clause.declares_vars() {
                ctx.declare(first_var.name(), key_ty);
                ctx.declare(second_var.name(), value_ty.clone());
            }
        }
        (Some(first_var), None) => {
            if range_clause.assigns_vars() {
                ctx.assign(first_var.name(), value_ty.clone());
            } else if range_clause.declares_vars() {
                ctx.declare(first_var.name(), value_ty.clone());
            }
        }
        _ => {}
    }

    // Check the loop body.
    let body_block = check_action_list(BlockKind::LoopBody, value_ty, range_list);

    let mut range_header_and_body_block = ctx.exit_block();
    range_header_and_body_block.merge_child(body_block);
    range_header_and_body_block
}

fn infer_iteration_var_types(ctx: &mut TypeckContext, range_expr_ty: &Ty) -> (Ty, Ty) {
    let range_expr_ty = base_ty(range_expr_ty, ctx.env);
    match range_expr_ty {
        Ty::Any => (Ty::Any, Ty::Any),

        Ty::Map(h) => {
            let map = &ctx.env.map_types[*h];
            (map.key_ty.clone(), map.value_ty.clone())
        }
        Ty::StaticStrMap(h) => {
            let static_str_map = &ctx.env.static_str_map_types[*h];
            (
                Ty::Primitive(PrimitiveTy::String),
                union_all(static_str_map.fields.values().map(|field| &field.ty)),
            )
        }
        Ty::Slice(h) => {
            let slice = &ctx.env.slice_types[*h];
            (Ty::Primitive(PrimitiveTy::Int), slice.el_ty.clone())
        }

        Ty::Primitive(primitive) if primitive.class() == PrimitiveClass::Integer => {
            (Ty::Primitive(PrimitiveTy::Int), Ty::Primitive(PrimitiveTy::Int))
        }

        ty => {
            // TODO: issue error
            (Ty::Any, Ty::Any)
        }
    }
}
