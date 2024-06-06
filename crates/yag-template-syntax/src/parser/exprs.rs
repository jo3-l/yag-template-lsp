use crate::parser::token_set::{TokenSet, ACTION_DELIMS, LEFT_DELIMS, RIGHT_DELIMS};
use crate::parser::{Checkpoint, Parser};
use crate::SyntaxKind;

pub(crate) fn expr(p: &mut Parser, context: &str) {
    const EXPR_RECOVERY_SET: TokenSet = ACTION_DELIMS.add(SyntaxKind::RightParen);

    let saw_dot = p.at(SyntaxKind::Dot);
    let c = p.checkpoint();
    match p.cur() {
        SyntaxKind::LeftParen => parenthesized(p),
        SyntaxKind::Ident => func_call(p, true),
        SyntaxKind::Dot => context_access_or_field_chain(p),
        SyntaxKind::Int | SyntaxKind::Bool => p.eat(),
        SyntaxKind::Var => {
            var(p);
        }
        SyntaxKind::InvalidChar => return, // lexer should have already emitted an error
        _ => return p.err_recover(format!("expected expression {context}"), EXPR_RECOVERY_SET),
    }

    // issue error for two dots in a row, eg in `..Field`, since
    // maybe_trailing_field_chain will interpret this construct like `(.).Field`
    // and does not error
    if saw_dot && p.at(SyntaxKind::Dot) {
        p.error_here("expected identifier");
    }
    maybe_trailing_field_chain(p, c);
    maybe_trailing_call_args(p, c);
    maybe_pipeline(p, c);
}

pub(crate) fn atom(p: &mut Parser) {
    let saw_dot = p.at(SyntaxKind::Dot);
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
        SyntaxKind::Dot => context_access_or_field_chain(p),
        SyntaxKind::Int | SyntaxKind::Bool => p.eat(),
        SyntaxKind::Var => {
            p.eat();
            p.wrap(c, SyntaxKind::VarAccess);
        }
        SyntaxKind::InvalidChar => return, // lexer should have already emitted an error
        _ => {
            // "expected atom" is not great end-user ux, so lie a little
            return p.err_recover("expected expression", ACTION_DELIMS);
        }
    }

    if saw_dot && p.at(SyntaxKind::Dot) {
        p.error_here("expected identifier");
    }
    maybe_trailing_field_chain(p, c);
}

pub(crate) fn maybe_trailing_field_chain(p: &mut Parser, c: Checkpoint) {
    let mut num_fields = 0;
    while p.at(SyntaxKind::Dot) {
        field(p);
        num_fields += 1;
    }
    if num_fields > 0 {
        p.wrap(c, SyntaxKind::ExprFieldChain);
    }
}

const CALL_TERMINATORS: TokenSet = LEFT_DELIMS
    .union(RIGHT_DELIMS)
    .add(SyntaxKind::Pipe)
    .add(SyntaxKind::RightParen)
    .add(SyntaxKind::Eof);

pub(crate) fn maybe_trailing_call_args(p: &mut Parser, c: Checkpoint) {
    let mut num_args = 0;
    while !p.at_ignore_space(CALL_TERMINATORS) {
        p.eat_whitespace();
        atom(p);
        num_args += 1;
    }
    if num_args > 0 {
        p.wrap(c, SyntaxKind::ExprCall);
    }
}

pub(crate) fn maybe_pipeline(p: &mut Parser, c: Checkpoint) {
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
    expr(p, "after `|`");
    pipeline_stage.complete(p);
}

pub(crate) fn parenthesized(p: &mut Parser) {
    let parenthesized = p.start(SyntaxKind::ParenthesizedExpr);
    p.expect(SyntaxKind::LeftParen);
    p.eat_whitespace();
    expr(p, "after `(`");
    p.eat_whitespace();
    p.expect_recover(SyntaxKind::RightParen, LEFT_DELIMS);
    parenthesized.complete(p);
}

pub(crate) fn func_call(p: &mut Parser, accept_args: bool) {
    let func_call = p.start(SyntaxKind::FuncCall);
    p.expect(SyntaxKind::Ident);

    if accept_args {
        while !p.at_ignore_space(CALL_TERMINATORS) {
            if !p.eat_whitespace() {
                p.error_here("expected whitespace between arguments");
            }
            atom(p);
        }
    }
    func_call.complete(p);
}

pub(crate) fn context_access_or_field_chain(p: &mut Parser) {
    let c = p.checkpoint();
    p.expect(SyntaxKind::Dot);
    // are we in a context field chain?
    if p.eat_if(SyntaxKind::Ident) {
        p.wrap(c, SyntaxKind::Field); // `.ident` is first field
        while p.at(SyntaxKind::Dot) {
            field(p);
        }
        p.wrap(c, SyntaxKind::ContextFieldChain);
    } else {
        // this is just a lone `.` accessing the context
        p.wrap(c, SyntaxKind::ContextAccess);
    }
}

pub(crate) fn field(p: &mut Parser) {
    let field = p.start(SyntaxKind::Field);
    p.expect(SyntaxKind::Dot);
    p.expect_recover(SyntaxKind::Ident, ACTION_DELIMS);
    field.complete(p);
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
            expr(p, "after `:=`");
            p.wrap(c, SyntaxKind::VarDecl);
        }
        SyntaxKind::Eq => {
            if saw_var && !saw_space_after_var {
                p.error_here("space required before `=` in assignment")
            }
            expr(p, "after `=`");
        }
        _ => (),
    }
}
