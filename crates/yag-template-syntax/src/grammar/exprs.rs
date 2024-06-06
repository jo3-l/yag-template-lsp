use crate::grammar::token_sets::{LEFT_DELIMS, RIGHT_DELIMS};
use crate::parser::{Checkpoint, Parser};
use crate::token_set::{token_set, TokenSet};
use crate::SyntaxKind;

pub(crate) fn expr(p: &mut Parser, atomic: bool) {
    let mut recoverable = LEFT_DELIMS.union(RIGHT_DELIMS);
    if !atomic {
        recoverable = recoverable.add(SyntaxKind::Pipe);
    }

    let c = p.checkpoint();
    match p.cur() {
        SyntaxKind::LeftParen => parenthesized(p),
        SyntaxKind::Ident => func_call(p, atomic),
        SyntaxKind::Int => p.eat(),
        SyntaxKind::Bool => p.eat(),
        SyntaxKind::Var => var(p, atomic),

        // try to recover from missing variable name in declaration/assignment: `{{ := 5}}`
        SyntaxKind::Eq | SyntaxKind::ColonEq if !atomic => var(p, false),
        _ => p.error_with_recover("expected expression", recoverable),
    }
    if !atomic && p.at(SyntaxKind::Pipe) {
        pipeline(p, c);
    }
}

pub(crate) fn pipeline(p: &mut Parser, c: Checkpoint) {
    while p.at(SyntaxKind::Pipe) {
        pipeline_stage(p);
    }
    p.wrap(c, SyntaxKind::Pipeline);
}

pub(crate) fn pipeline_stage(p: &mut Parser) {
    let pipeline_stage = p.start(SyntaxKind::PipelineStage);
    p.expect(SyntaxKind::Pipe);
    if p.at(SyntaxKind::Ident) {
        func_call(p, false);
    } else {
        p.error_with_recover(
            "expected function call after pipe",
            LEFT_DELIMS.union(RIGHT_DELIMS).add(SyntaxKind::RightParen),
        );
    }
    pipeline_stage.complete(p);
}

pub(crate) fn parenthesized(p: &mut Parser) {
    let parenthesized = p.start(SyntaxKind::ParenthesizedExpr);
    p.expect(SyntaxKind::LeftParen);
    expr(p, false);
    p.expect_with_recover(SyntaxKind::RightParen, LEFT_DELIMS);
    parenthesized.complete(p);
}

pub(crate) fn func_call(p: &mut Parser, atomic: bool) {
    const EXPR_TERMINATORS: TokenSet = LEFT_DELIMS
        .union(RIGHT_DELIMS)
        .add(SyntaxKind::Pipe)
        .add(SyntaxKind::RightParen)
        .add(SyntaxKind::Eof);

    let func_call = p.start(SyntaxKind::FuncCall);
    p.expect(SyntaxKind::Ident);
    if !atomic {
        while !p.at(EXPR_TERMINATORS) {
            if !p.preceded_by_whitespace() {
                p.error(
                    "expected whitespace before function call argument",
                    p.cur_range(),
                );
            }
            expr(p, true);
        }
    }
    func_call.complete(p);
}

pub(crate) fn var(p: &mut Parser, atomic: bool) {
    if atomic {
        let var_ref = p.start(SyntaxKind::VarRef);
        p.expect(SyntaxKind::Var);
        var_ref.complete(p);
        return;
    }

    let c = p.checkpoint();
    p.expect_with_recover(SyntaxKind::Var, token_set! { ColonEq, Eq });
    match p.cur() {
        SyntaxKind::ColonEq => {
            p.eat();
            expr(p, false);
            p.wrap(c, SyntaxKind::VarDecl);
        }
        SyntaxKind::Eq => {
            if p.preceded_by_whitespace() {
                p.eat()
            } else {
                p.error_and_eat("expected whitespace before equals sign")
            }
            expr(p, false);
            p.wrap(c, SyntaxKind::VarAssign);
        }
        _ => (),
    }
}
