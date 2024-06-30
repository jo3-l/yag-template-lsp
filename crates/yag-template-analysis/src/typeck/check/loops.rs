use yag_template_syntax::ast::{self, AstNode, SyntaxNodeExt};

use super::check_action_list;
use super::expr::check_expr;
use crate::typeck::context::TypeckContext;
use crate::typeck::flow::{Block, BlockKind};
use crate::typeck::ty::{base_ty, union_all, PrimitiveClass, PrimitiveTy, Ty};

pub(crate) fn check_while_loop(ctx: &mut TypeckContext, while_loop: ast::WhileLoop) -> Block {
    ctx.enter_block(BlockKind::default(), ctx.inherit_context_ty());

    let cond_expr = while_loop.while_clause().and_then(|clause| clause.cond_expr());
    check_expr(ctx, cond_expr.clone());

    let while_body_block = check_while_loop_body(ctx, cond_expr, while_loop.action_list());
    let while_else_block = check_action_list(
        ctx,
        BlockKind::default(),
        ctx.inherit_context_ty(),
        while_loop.else_branch().and_then(|branch| branch.action_list()),
    );

    let mut while_block = ctx.exit_block();
    while_block.merge_divergent_branches(while_body_block, while_else_block);
    while_block
}

fn check_while_loop_body(
    ctx: &mut TypeckContext,
    cond_expr: Option<ast::Expr>,
    body: Option<ast::ActionList>,
) -> Block {
    let mut while_body_block = check_action_list(ctx, BlockKind::LoopBody, ctx.inherit_context_ty(), body);
    // Pessimistically mark all variable assignments in the loop body that are also assigned in the
    // loop condition as indefinite. Otherwise, the type of variable $x following the loop in the
    // program:
    //
    //   {{while $x = funcReturningInt}}
    //     {{$x = "str"}}
    //   {{end}}
    //   {{$x}}
    //
    // would be `string` (whereas it is always `int` in reality.) Note that with the assumption
    // below, the actual type reported is not `int` but rather `int | string` to account for the
    // possibility of an immediate `break` following the assignment in the loop body:
    //
    //   {{while $x = funcReturningInt}}
    //     {{if cond}}
    //       {{$x = "str"}}
    //       {{break}}
    //     {{end}}
    //   {{end}}
    //   {{$x}}
    //
    // While it is probably possible to further refine the analysis in this case, assignments in
    // loop conditions are uncommon enough and a fix would likely require building a complete CFG,
    // so it is not worth the effort for now.
    for assigned_var in cond_expr.into_iter().flat_map(var_assigns_in) {
        while_body_block
            .var_assigns
            .entry(assigned_var.name().into())
            .and_modify(|assign| assign.is_definite = false);
    }
    while_body_block
}

fn var_assigns_in(expr: ast::Expr) -> impl Iterator<Item = ast::Var> {
    expr.syntax()
        .descendants()
        .filter_map(|node| node.try_to::<ast::VarAssign>()?.var())
}

pub(crate) fn check_range_loop(ctx: &mut TypeckContext, range: ast::RangeLoop) -> Block {
    ctx.enter_block(BlockKind::default(), ctx.inherit_context_ty());

    let range_header_and_body_block = check_range_header_and_body(ctx, range.range_clause(), range.action_list());
    let range_else_block = check_action_list(
        ctx,
        BlockKind::default(),
        ctx.inherit_context_ty(),
        range.else_branch().and_then(|branch| branch.action_list()),
    );

    let mut range_block = ctx.exit_block();
    range_block.merge_divergent_branches(range_header_and_body_block, range_else_block);
    range_block
}

fn check_range_header_and_body(
    ctx: &mut TypeckContext,
    range_clause: Option<ast::RangeClause>,
    range_list: Option<ast::ActionList>,
) -> Block {
    let Some(range_clause) = range_clause else {
        return Block::never();
    };
    ctx.enter_block(BlockKind::default(), ctx.inherit_context_ty());

    let range_expr_ty = check_expr(ctx, range_clause.range_expr());
    let (key_ty, value_ty) = key_value_types(ctx, &range_expr_ty);
    infer_iter_var_types(range_clause.iteration_vars(), &key_ty, &value_ty, |var, ty| {
        if range_clause.assigns_vars() {
            ctx.assign(var.name(), ty);
        } else if range_clause.declares_vars() {
            ctx.declare(var.name(), ty);
        }
    });

    let body_block = check_action_list(ctx, BlockKind::LoopBody, value_ty, range_list);

    let mut range_header_and_body_block = ctx.exit_block();
    range_header_and_body_block.merge(body_block);
    range_header_and_body_block
}

fn infer_iter_var_types<F>(mut iteration_vars: impl Iterator<Item = ast::Var>, key_ty: &Ty, value_ty: &Ty, mut visit: F)
where
    F: FnMut(ast::Var, Ty),
{
    match (iteration_vars.next(), iteration_vars.next()) {
        (Some(first_var), Some(second_var)) => {
            visit(first_var, key_ty.clone());
            visit(second_var, value_ty.clone());
        }
        (Some(first_var), None) => visit(first_var, value_ty.clone()),
        _ => {}
    }

    for additional_var in iteration_vars {
        visit(additional_var, Ty::Any);
    }
}

fn key_value_types(ctx: &mut TypeckContext, ty: &Ty) -> (Ty, Ty) {
    let ty = base_ty(ty, ctx.env);
    match ty {
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
