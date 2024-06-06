use super::token_sets::RIGHT_DELIMS;
use crate::grammar::exprs::expr;
use crate::grammar::token_sets::LEFT_DELIMS;
use crate::parser::Parser;
use crate::token_set::TokenSet;
use crate::{token_set, SyntaxKind};

pub(crate) fn action_list(p: &mut Parser) {
    const TERMINATORS: TokenSet = token_set! { End };

    let m = p.marker();
    while !p.done() && !p.at2(LEFT_DELIMS, TERMINATORS) {
        text_or_action(p);
    }
    p.wrap(m, SyntaxKind::ActionList);
}

pub(crate) fn text_or_action(p: &mut Parser) {
    if p.eat_if(SyntaxKind::Text) {
        return;
    }

    if !p.at(LEFT_DELIMS) {
        return p.error_and_eat("expected left action delimiter");
    }
    match p.peek() {
        SyntaxKind::If => if_action(p),
        _ => expr_action(p),
    }
}

pub(crate) fn if_action(p: &mut Parser) {
    let m = p.marker();
    if_clause(p);
    action_list(p);
    end_clause(p);
    p.wrap(m, SyntaxKind::If);
}

pub(crate) fn if_clause(p: &mut Parser) {
    let m = p.marker();
    left_delim(p);
    p.expect_with_recover(SyntaxKind::If, LEFT_DELIMS);
    expr(p, false);
    right_delim(p);
    p.wrap(m, SyntaxKind::IfClause);
}

pub(crate) fn end_clause(p: &mut Parser) {
    let m = p.marker();
    left_delim(p);
    p.expect_with_recover(SyntaxKind::End, LEFT_DELIMS);
    right_delim(p);
    p.wrap(m, SyntaxKind::EndClause);
}

pub(crate) fn expr_action(p: &mut Parser) {
    let m = p.marker();
    left_delim(p);
    expr(p, false);
    right_delim(p);
    p.wrap(m, SyntaxKind::ExprAction);
}

pub(crate) fn left_delim(p: &mut Parser) {
    if !p.eat_if(LEFT_DELIMS) {
        p.error_and_eat("expected left action delimiter");
    }
}

pub(crate) fn right_delim(p: &mut Parser) {
    if !p.eat_if(RIGHT_DELIMS) {
        p.error_with_recover("expected right action delimiter", LEFT_DELIMS);
    }
}