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
    const FUNC_CALL_TERMINATORS: TokenSet = LEFT_DELIMS.union(RIGHT_DELIMS).add(SyntaxKind::Eof);

    let func_call = p.start(SyntaxKind::FuncCall);
    p.assert(SyntaxKind::Ident);
    if !atomic {
        while !p.at(FUNC_CALL_TERMINATORS) {
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
    let c = p.checkpoint();
    p.assert(SyntaxKind::Var);
    if atomic {
        return p.wrap(c, SyntaxKind::VarRef);
    }

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
