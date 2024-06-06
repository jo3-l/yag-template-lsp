use crate::parser::token_set::{TokenSet, ACTION_DELIMS, LEFT_DELIMS, RIGHT_DELIMS};
use crate::parser::{Checkpoint, Parser};
use crate::SyntaxKind;

pub(crate) fn expr(p: &mut Parser) {
    const EXPR_RECOVERY_SET: TokenSet = ACTION_DELIMS.add(SyntaxKind::RightParen);

    let c = p.checkpoint();
    match p.cur() {
        SyntaxKind::LeftParen => parenthesized(p),
        SyntaxKind::Ident => func_call(p, true),
        SyntaxKind::Int | SyntaxKind::Bool => p.eat(),
        SyntaxKind::Var => {
            var(p);
        }
        _ => {
            p.error_recover("expected expression", EXPR_RECOVERY_SET);
            return;
        }
    }

    maybe_wrap_in_pipeline(p, c);

    // field access
    // call args
    // pipeline
}

pub(crate) fn maybe_wrap_in_pipeline(p: &mut Parser, c: Checkpoint) {
    if !p.at_ignore_space(SyntaxKind::Pipe) {
        return;
    }

    while p.at_ignore_space(SyntaxKind::Pipe) {
        p.eat_whitespace();
        pipeline_stage(p);
    }
    p.wrap(c, SyntaxKind::Pipeline);
}

pub(crate) fn pipeline_stage(p: &mut Parser) {
    let pipeline_stage = p.start(SyntaxKind::PipelineStage);
    p.expect(SyntaxKind::Pipe);
    p.eat_whitespace();
    expr(p);
    pipeline_stage.complete(p);
}

pub(crate) fn atom(p: &mut Parser) {
    let c = p.checkpoint();
    match p.cur() {
        SyntaxKind::LeftParen => parenthesized(p),
        SyntaxKind::Ident => {
            // Don't eat call arguments in atom context:
            //   {{add currentHour 2}}
            // should be parsed as
            //   add(currentHour(), 2)
            // not
            //   add(currentHour(2)).
            func_call(p, false);
        }
        SyntaxKind::Int | SyntaxKind::Bool => p.eat(),
        SyntaxKind::Var => {
            p.eat();
            p.wrap(c, SyntaxKind::VarAccess);
        }
        _ => p.error_recover("expected expression", ACTION_DELIMS), // "expected atom" is not great end-user ux, so lie a little
    }

    // field access
}

pub(crate) fn parenthesized(p: &mut Parser) {
    let parenthesized = p.start(SyntaxKind::ParenthesizedExpr);
    p.expect(SyntaxKind::LeftParen);
    p.eat_whitespace();
    expr(p);
    p.eat_whitespace();
    p.expect_recover(SyntaxKind::RightParen, LEFT_DELIMS);
    parenthesized.complete(p);
}

pub(crate) fn func_call(p: &mut Parser, accept_args: bool) {
    const CALL_TERMINATORS: TokenSet = LEFT_DELIMS
        .union(RIGHT_DELIMS)
        .add(SyntaxKind::Pipe)
        .add(SyntaxKind::RightParen)
        .add(SyntaxKind::Eof);

    let func_call = p.start(SyntaxKind::FuncCall);
    p.expect(SyntaxKind::Ident);

    if accept_args {
        while !p.at_ignore_space(CALL_TERMINATORS) {
            if !p.eat_whitespace() {
                p.error("expected whitespace between arguments", p.cur_range());
            }
            atom(p);
        }
    }
    func_call.complete(p);
}

pub(crate) fn var(p: &mut Parser) {
    const DECLARE_ASSIGN_OPS: TokenSet =
        TokenSet::new().add(SyntaxKind::ColonEq).add(SyntaxKind::Eq);

    let c = p.checkpoint();

    // gracefully handle declarations and assignments with missing variable names
    let saw_var = p.expect_recover(SyntaxKind::Var, DECLARE_ASSIGN_OPS);
    if saw_var && !DECLARE_ASSIGN_OPS.contains(p.peek_non_space()) {
        p.wrap(c, SyntaxKind::VarAccess);
    }

    let saw_space_after_var = p.eat_whitespace();
    match p.cur() {
        SyntaxKind::ColonEq => {
            p.eat();
            expr(p);
            p.wrap(c, SyntaxKind::VarDecl);
        }
        SyntaxKind::Eq => {
            if saw_var && !saw_space_after_var {
                p.error_here("space required before `=` in assignment")
            }
            expr(p);
        }
        _ => (),
    }
}
