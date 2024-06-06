use super::token_sets::RIGHT_DELIMS;
use crate::grammar::exprs::expr;
use crate::grammar::token_sets::LEFT_DELIMS;
use crate::parser::Parser;
use crate::token_set::{token_set, TokenSet};
use crate::SyntaxKind;

pub(crate) fn action_list(p: &mut Parser) {
    const TERMINATORS: TokenSet = token_set! { Else, End };

    let action_list = p.start(SyntaxKind::ActionList);
    while !p.done() && !p.at2(LEFT_DELIMS, TERMINATORS) {
        text_or_action(p);
    }
    action_list.complete(p);
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
    let if_action = p.start(SyntaxKind::IfConditional);
    if_clause(p);
    action_list(p);
    while p.at2(LEFT_DELIMS, SyntaxKind::Else) {
        else_branch(p);
    }
    end_clause(p, "if action");
    if_action.complete(p);
}

pub(crate) fn if_clause(p: &mut Parser) {
    let if_clause = p.start(SyntaxKind::IfClause);
    left_delim(p);
    p.expect_with_recover(SyntaxKind::If, LEFT_DELIMS);
    expr(p, false);
    right_delim(p);
    if_clause.complete(p);
}

pub(crate) fn else_branch(p: &mut Parser) {
    let else_branch = p.start(SyntaxKind::ElseBranch);
    else_clause(p);
    action_list(p);
    else_branch.complete(p);
}

pub(crate) fn else_clause(p: &mut Parser) {
    let else_clause = p.start(SyntaxKind::ElseClause);
    left_delim(p);
    p.expect(SyntaxKind::Else);
    match p.cur() {
        SyntaxKind::RightDelim | SyntaxKind::TrimmedRightDelim => p.eat(),
        SyntaxKind::If => {
            p.eat();
            expr(p, false);
            right_delim(p);
        }
        _ => p.error_with_recover(
            "expected expression or right action delimiter after `else` keyword",
            LEFT_DELIMS,
        ),
    }
    else_clause.complete(p);
}

pub(crate) fn end_clause(p: &mut Parser, context: &str) {
    if !p.at2(LEFT_DELIMS, SyntaxKind::End) {
        p.error_with_recover(format!("missing end clause for {context}"), LEFT_DELIMS);
        return;
    }

    let end_clause = p.start(SyntaxKind::EndClause);
    left_delim(p);
    p.expect(SyntaxKind::End);
    right_delim(p);
    end_clause.complete(p);
}

pub(crate) fn expr_action(p: &mut Parser) {
    let expr_action = p.start(SyntaxKind::ExprAction);
    left_delim(p);
    expr(p, false);
    right_delim(p);
    expr_action.complete(p);
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
