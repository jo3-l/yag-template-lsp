use crate::parser::token_set::{TokenSet, ACTION_DELIMS, LEFT_DELIMS, RIGHT_DELIMS};
use crate::parser::{Checkpoint, Parser};
use crate::SyntaxKind;

pub(crate) fn expr(p: &mut Parser, context: &str) {
    const EXPR_RECOVERY_SET: TokenSet = ACTION_DELIMS.add(SyntaxKind::RightParen);

    let c = p.checkpoint();
    let saw_dot = p.at(SyntaxKind::Dot);
    match p.cur() {
        SyntaxKind::LeftParen => parenthesized(p),
        SyntaxKind::Ident => func_call(p, true),
        SyntaxKind::Field => context_field_chain(p),
        SyntaxKind::Dot => context_access(p),
        SyntaxKind::Int | SyntaxKind::Bool => p.eat(),
        SyntaxKind::Var => var(p),

        SyntaxKind::InvalidChar => p.eat(), // lexer should have already emitted an error; don't duplicate
        _ => {
            return p.err_recover(
                format!("expected expression {context}; found {}", p.cur_name()),
                EXPR_RECOVERY_SET,
            )
        }
    }

    // issue error for two dots in a row, eg in `..Field`, since
    // maybe_trailing_field_chain will interpret this construct as `(.).Field`
    // and does not error
    if saw_dot && p.at(SyntaxKind::Field) {
        p.error_here("expected identifier after `.`");
    }
    maybe_wrap_trailing_field_chain(p, c);
    maybe_wrap_trailing_call_args(p, c);
    maybe_wrap_pipeline(p, c);
}

pub(crate) fn atom(p: &mut Parser) {
    const ATOM_RECOVERY_SET: TokenSet = ACTION_DELIMS.add(SyntaxKind::RightParen);

    let saw_dot = p.at(SyntaxKind::Dot);
    let c = p.checkpoint();
    match p.cur() {
        SyntaxKind::LeftParen => parenthesized(p),
        // Don't eat call arguments in atom context:
        //   {{add currentHour 2}}
        // should be parsed as
        //   add(currentHour(), 2)
        // not
        //   add(currentHour(2)).
        SyntaxKind::Ident => func_call(p, false),
        SyntaxKind::Field => context_field_chain(p),
        SyntaxKind::Dot => context_access(p),
        SyntaxKind::Int | SyntaxKind::Bool => p.eat(),
        SyntaxKind::Var => {
            p.eat();
            p.wrap(c, SyntaxKind::VarAccess);
        }

        SyntaxKind::InvalidChar => p.eat(), // lexer should have already emitted an error; don't duplicate
        _ => {
            // "expected atom" is not great end-user ux, so lie a little
            return p.err_recover(
                format!("expected expression; found {}", p.cur_name()),
                ATOM_RECOVERY_SET,
            );
        }
    }

    if saw_dot && p.at(SyntaxKind::Dot) {
        p.error_here("expected identifier");
    }
    maybe_wrap_trailing_field_chain(p, c);
}

pub(crate) fn maybe_wrap_trailing_field_chain(p: &mut Parser, c: Checkpoint) {
    let num_fields = eat_fields(p);
    if num_fields > 0 {
        p.wrap(c, SyntaxKind::ExprFieldChain);
    }
}

pub(crate) fn eat_fields(p: &mut Parser) -> usize {
    let mut num_fields = 0;
    loop {
        match p.cur() {
            SyntaxKind::Field => p.eat(),
            // handle missing field name as in `.Field1.Field2.` gracefully
            SyntaxKind::Dot => {
                let field = p.start(SyntaxKind::Field);
                p.err_and_eat("expected field name after `.`");
                field.complete(p);
            }
            _ => return num_fields,
        }
        num_fields += 1;
    }
}

const CALL_TERMINATORS: TokenSet = LEFT_DELIMS
    .union(RIGHT_DELIMS)
    .add(SyntaxKind::Pipe)
    .add(SyntaxKind::RightParen)
    .add(SyntaxKind::Eof);

pub(crate) fn maybe_wrap_trailing_call_args(p: &mut Parser, c: Checkpoint) {
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

pub(crate) fn maybe_wrap_pipeline(p: &mut Parser, c: Checkpoint) {
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
        let mut num_args = 0;
        while !p.at_ignore_space(CALL_TERMINATORS) {
            if !p.eat_whitespace() {
                p.error_here(if num_args > 0 {
                    "expected space between function arguments"
                } else {
                    "expected space separating function name and argument"
                });
            }
            atom(p);
            num_args += 1;
        }
    }
    func_call.complete(p);
}

pub(crate) fn context_access(p: &mut Parser) {
    let context_access = p.start(SyntaxKind::ContextAccess);
    p.expect(SyntaxKind::Dot);
    context_access.complete(p);
}

pub(crate) fn context_field_chain(p: &mut Parser) {
    let context_field_chain = p.start(SyntaxKind::ContextFieldChain);
    eat_fields(p);
    context_field_chain.complete(p);
}

pub(crate) fn var(p: &mut Parser) {
    const DECLARE_ASSIGN_OPS: TokenSet = TokenSet::new().add(SyntaxKind::ColonEq).add(SyntaxKind::Eq);

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
                p.error_here("space required before `=` in assignment");
            }
            expr(p, "after `=`");
        }
        _ => (),
    }
}
