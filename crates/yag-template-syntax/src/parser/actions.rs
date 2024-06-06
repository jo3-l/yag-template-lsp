use super::token_set::{ACTION_DELIMS, LEFT_DELIMS, RIGHT_DELIMS};
use super::TokenPattern;
use crate::parser::exprs::expr;
use crate::parser::token_set::TokenSet;
use crate::parser::Parser;
use crate::SyntaxKind;

impl Parser<'_> {
    pub(crate) fn at_left_delim_and(&mut self, pat: impl TokenPattern) -> bool {
        self.at(LEFT_DELIMS) && pat.matches(self.peek_non_space())
    }
}

pub(crate) fn action_list(p: &mut Parser) {
    const ACTION_LIST_TERMINATORS: TokenSet =
        TokenSet::new().add(SyntaxKind::End).add(SyntaxKind::Else);

    let action_list = p.start(SyntaxKind::ActionList);
    // until EOF, `{{end`, or `{{else`
    while !p.at_eof() && !p.at_left_delim_and(ACTION_LIST_TERMINATORS) {
        text_or_action(p);
    }
    action_list.complete(p);
}

pub(crate) fn text_or_action(p: &mut Parser) {
    if p.eat_if(SyntaxKind::Text) {
        return;
    }

    if !p.at(LEFT_DELIMS) {
        return p.err_and_eat("expected left action delimiter");
    }
    match p.peek_non_space() {
        SyntaxKind::If => if_action(p),
        _ => expr_action(p),
    }
}

pub(crate) fn if_action(p: &mut Parser) {
    let if_action = p.start(SyntaxKind::IfConditional);
    if_clause(p);
    action_list(p);
    while p.at_left_delim_and(SyntaxKind::Else) {
        else_branch(p);
    }
    end_clause(p, "if action");
    if_action.complete(p);
}

pub(crate) fn if_clause(p: &mut Parser) {
    let if_clause = p.start(SyntaxKind::IfClause);
    left_delim(p);
    p.eat_whitespace();

    let saw_if_kw = p.expect(SyntaxKind::If);
    if saw_if_kw && !p.eat_whitespace() {
        p.error_here("expected space between `if` keyword and condition");
    }

    expr(p, "after `if` keyword");
    p.eat_whitespace();
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
    p.eat_whitespace();
    match p.cur() {
        SyntaxKind::RightDelim | SyntaxKind::TrimmedRightDelim => p.eat(),
        SyntaxKind::If => {
            p.eat();
            expr(p, "after `else if`");
            p.eat_whitespace();
            right_delim(p);
        }
        _ => p.err_recover(
            "expected `if` keyword or right action delimiter after `else` keyword",
            LEFT_DELIMS,
        ),
    }
    else_clause.complete(p);
}

pub(crate) fn end_clause(p: &mut Parser, parent_context: &str) {
    if !p.at_left_delim_and(SyntaxKind::End) {
        p.err_recover(
            format!("missing end clause for {parent_context}"),
            LEFT_DELIMS,
        );
        return;
    }

    let end_clause = p.start(SyntaxKind::EndClause);
    left_delim(p);
    p.eat_whitespace();
    p.expect(SyntaxKind::End);
    p.eat_whitespace();
    right_delim(p);
    end_clause.complete(p);
}

pub(crate) fn expr_action(p: &mut Parser) {
    let expr_action = p.start(SyntaxKind::ExprAction);
    left_delim(p);
    p.eat_whitespace();
    expr(p, "after `{{`");
    p.eat_whitespace();
    right_delim(p);
    expr_action.complete(p);
}

pub(crate) fn left_delim(p: &mut Parser) {
    if !p.eat_if(LEFT_DELIMS) {
        p.err_and_eat("expected left action delimiter");
    }
}

pub(crate) fn right_delim(p: &mut Parser) {
    while !p.at(ACTION_DELIMS) {
        if p.eat_if(SyntaxKind::InvalidChar) {
            // lexer should already have emitted an error
        } else {
            p.err_and_eat(format!("unexpected {} in action", p.cur().name()))
        }
    }

    if !p.eat_if(RIGHT_DELIMS) {
        p.err_recover("expected right action delimiter", LEFT_DELIMS);
    }
}
