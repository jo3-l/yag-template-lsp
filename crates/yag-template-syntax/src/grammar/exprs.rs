use crate::grammar::token_sets::{LEFT_DELIMS, RIGHT_DELIMS};
use crate::parser::Parser;
use crate::token_set::TokenSet;
use crate::SyntaxKind;

pub(crate) fn expr(p: &mut Parser, atomic: bool) {
    match p.cur() {
        SyntaxKind::Ident => func_call(p, atomic),
        SyntaxKind::Int => p.eat(),
        SyntaxKind::Bool => p.eat(),
        SyntaxKind::Var => var(p, atomic),
        _ => p.error_with_recover("expected expression", LEFT_DELIMS),
    }
}

pub(crate) fn func_call(p: &mut Parser, atomic: bool) {
    const FUNC_CALL_TERMINATOR: TokenSet = LEFT_DELIMS.union(RIGHT_DELIMS).add(SyntaxKind::Eof);

    let m = p.marker();
    p.assert(SyntaxKind::Ident);
    if !atomic {
        while !p.at(FUNC_CALL_TERMINATOR) {
            if !p.had_leading_whitespace() {
                p.error(
                    "expected whitespace before function call argument",
                    p.cur_range(),
                );
            }
            expr(p, true);
        }
    }
    p.wrap(m, SyntaxKind::FuncCall);
}

pub(crate) fn var(p: &mut Parser, atomic: bool) {
    let m = p.marker();
    p.assert(SyntaxKind::Var);
    if atomic {
        return p.wrap(m, SyntaxKind::VarRef);
    }

    match p.cur() {
        SyntaxKind::ColonEq => {
            p.eat();
            expr(p, false);
            p.wrap(m, SyntaxKind::VarDecl);
        }
        SyntaxKind::Eq => {
            if p.had_leading_whitespace() {
                p.eat()
            } else {
                p.error_and_eat("expected whitespace before equals sign")
            }
            expr(p, false);
            p.wrap(m, SyntaxKind::VarAssign);
        }
        _ => p.abandon(m),
    }
}
