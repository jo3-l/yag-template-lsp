use crate::parser::token_set::{TokenSet, ACTION_DELIMS, LEFT_DELIMS};
use crate::parser::{Checkpoint, Parser};
use crate::SyntaxKind;

/// Parse a pipeline of expressions. (If there is only one expression,
/// `expr_pipeline` behaves identically to `expr`.)
pub(crate) fn expr_pipeline(p: &mut Parser, context: &str) {
    let c = p.checkpoint();
    expr(p, context);
    if p.at_ignore_space(SyntaxKind::Pipe) {
        while p.at_ignore_space(SyntaxKind::Pipe) {
            p.eat_whitespace();
            pipeline_stage(p);
        }
        p.wrap(c, SyntaxKind::Pipeline);
    }
}

fn pipeline_stage(p: &mut Parser) {
    let pipeline_stage = p.start(SyntaxKind::PipelineStage);
    p.expect(SyntaxKind::Pipe);
    p.eat_whitespace();
    expr(p, "after `|`");
    pipeline_stage.complete(p);
}

/// Parse an expression, possibly with trailing field chain and call arguments.
pub(crate) fn expr(p: &mut Parser, context: &str) {
    const EXPR_RECOVERY_SET: TokenSet = ACTION_DELIMS.add(SyntaxKind::RightParen);

    let c = p.checkpoint();
    let saw_dot = p.at(SyntaxKind::Dot);
    match p.cur() {
        SyntaxKind::LeftParen => parenthesized(p),
        SyntaxKind::Ident => func_call(p, true),
        SyntaxKind::Field => context_field_chain(p),
        SyntaxKind::Dot => context_access(p),
        SyntaxKind::Var => var(p),
        token if token.is_literal() => literal(p),

        SyntaxKind::InvalidCharInAction => p.eat(), // lexer should have already emitted an error; don't duplicate
        token => p.err_recover(
            format!("expected expression {context}; found {token}"),
            EXPR_RECOVERY_SET,
        ),
    }

    // issue error for two dots in a row: `..Field`
    if saw_dot && (p.at(SyntaxKind::Field) || p.at(SyntaxKind::Dot)) {
        p.error_here("expected field name after `.`");
    }
    trailing_field_chain(p, c);
    trailing_call_args(p, c);
}

/// Parse an argument to a call.
pub(crate) fn arg(p: &mut Parser) {
    const ARG_RECOVERY_SET: TokenSet = ACTION_DELIMS.add(SyntaxKind::RightParen);

    let saw_dot = p.at(SyntaxKind::Dot);
    let c = p.checkpoint();
    match p.cur() {
        SyntaxKind::LeftParen => parenthesized(p),
        // Don't accept additional arguments:
        //   {{add currentHour 2}}
        // should be parsed as
        //   add(currentHour(), 2)
        // not
        //   add(currentHour(2)).
        SyntaxKind::Ident => func_call(p, false),
        SyntaxKind::Field => context_field_chain(p),
        SyntaxKind::Dot => context_access(p),
        // Variables in argument position can only
        // be variable accesses, not assignments or declarations:
        //   {{add $x := 2 3}}
        // is invalid.
        SyntaxKind::Var => {
            p.eat();
            p.wrap(c, SyntaxKind::VarAccess);
        }
        token if token.is_literal() => literal(p),

        SyntaxKind::InvalidCharInAction => p.eat(), // lexer should have already emitted an error; don't duplicate
        token => p.err_recover(format!("expected argument; found {token}"), ARG_RECOVERY_SET),
    }

    if saw_dot && (p.at(SyntaxKind::Field) || p.at(SyntaxKind::Dot)) {
        p.error_here("expected field name after `.`");
    }
    trailing_field_chain(p, c);
}

fn trailing_field_chain(p: &mut Parser, c: Checkpoint) {
    let num_fields = eat_fields(p);
    if num_fields > 0 {
        p.wrap(c, SyntaxKind::ExprFieldChain);
    }
}

fn eat_fields(p: &mut Parser) -> usize {
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

fn at_call_terminator(p: &mut Parser) -> bool {
    const SPACE_INDEPENDENT_TERMINATORS: TokenSet = ACTION_DELIMS
        .add(SyntaxKind::Pipe)
        .add(SyntaxKind::RightParen)
        .add(SyntaxKind::Eof);

    p.at_ignore_space(SPACE_INDEPENDENT_TERMINATORS)
        // func.Field has no arguments, but func .Field has 1 argument
        || p.at(SyntaxKind::Field)
}

fn trailing_call_args(p: &mut Parser, c: Checkpoint) {
    let mut num_args = 0;
    while !at_call_terminator(p) {
        if num_args > 0 {
            p.expect_whitespace("between call arguments");
        } else {
            p.expect_whitespace("separating expression and call arguments");
        }
        arg(p);
        num_args += 1;
    }
    if num_args > 0 {
        p.wrap(c, SyntaxKind::ExprCall);
    }
}

fn parenthesized(p: &mut Parser) {
    let parenthesized = p.start(SyntaxKind::ParenthesizedExpr);
    p.expect(SyntaxKind::LeftParen);
    p.eat_whitespace();
    expr_pipeline(p, "after `(`");
    p.eat_whitespace();
    p.expect_recover(SyntaxKind::RightParen, LEFT_DELIMS);
    parenthesized.complete(p);
}

fn func_call(p: &mut Parser, accept_args: bool) {
    let func_call = p.start(SyntaxKind::FuncCall);
    p.expect(SyntaxKind::Ident);

    if accept_args {
        let mut num_args = 0;
        while !at_call_terminator(p) {
            if num_args > 0 {
                p.expect_whitespace("between function arguments");
            } else {
                p.expect_whitespace("separating function name and argument");
            }
            arg(p);
            num_args += 1;
        }
    }
    func_call.complete(p);
}

fn context_access(p: &mut Parser) {
    let context_access = p.start(SyntaxKind::ContextAccess);
    p.expect(SyntaxKind::Dot);
    context_access.complete(p);
}

fn context_field_chain(p: &mut Parser) {
    let context_field_chain = p.start(SyntaxKind::ContextFieldChain);
    eat_fields(p);
    context_field_chain.complete(p);
}

fn var(p: &mut Parser) {
    const DECLARE_ASSIGN_OPS: TokenSet = TokenSet::new().add(SyntaxKind::ColonEq).add(SyntaxKind::Eq);

    let c = p.checkpoint();

    // recover from missing variable name
    p.expect_recover(SyntaxKind::Var, DECLARE_ASSIGN_OPS);

    if p.at_ignore_space(SyntaxKind::ColonEq) {
        // $x := expr assignment; whitespace is optional
        p.eat_whitespace();
        p.expect(SyntaxKind::ColonEq);
        p.eat_whitespace();
        expr(p, "after `:=`");
        p.wrap(c, SyntaxKind::VarDecl);
    } else if p.at_ignore_space(SyntaxKind::Eq) {
        // $x = expr; must have whitespace between the variable and the `=`
        // symbol but not necessarily after the `=`
        p.expect_whitespace("before `=` in assignment");
        p.expect(SyntaxKind::Eq);
        p.eat_whitespace();
        expr(p, "after `=`");
        p.wrap(c, SyntaxKind::VarAssign);
    } else {
        // neither followed by `:=` nor by `=`, so treat as normal variable
        // access
        p.wrap(c, SyntaxKind::VarAccess);
    }
}

fn literal(p: &mut Parser) {
    let literal = p.start(SyntaxKind::Literal);
    p.eat();
    literal.complete(p);
}
