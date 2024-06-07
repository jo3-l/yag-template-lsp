use std::io::{self, Write};

use rowan::TextRange;

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
    const ACTION_LIST_TERMINATORS: TokenSet = TokenSet::new().add(SyntaxKind::End).add(SyntaxKind::Else);

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
        return p.err_and_eat(format!("expected left action delimiter; found {}", p.cur_name()));
    }
    match p.peek_non_space() {
        SyntaxKind::If => if_conditional(p),
        SyntaxKind::Range => range_loop(p),
        _ => expr_action(p),
    }
}

pub(crate) fn if_conditional(p: &mut Parser) {
    let if_conditional = p.start(SyntaxKind::IfConditional);
    if_clause(p);
    action_list(p);
    else_branches(p, "if action", true);
    end_clause(p, "if action");
    if_conditional.complete(p);
}

pub(crate) fn if_clause(p: &mut Parser) {
    let if_clause = p.start(SyntaxKind::IfClause);
    left_delim(p);
    p.eat_whitespace();
    p.expect(SyntaxKind::If);
    if !p.eat_whitespace() {
        p.error_here(format!("expected space after `if` keyword; found {}", p.cur_name()));
    }
    expr(p, "after `if` keyword");
    p.eat_whitespace();
    right_delim(p);
    if_clause.complete(p);
}

pub(crate) fn else_branches(p: &mut Parser, parent_context: &str, permit_else_if: bool) {
    // Make sure we issue an error if any additional {{else if}} or {{else}}
    // branches appear after the first unconditional {{else}} branch.
    //
    // Specifically, both of the following are syntax errors:
    //   ... {{else}} ... {{else if}} ... {{end}}
    //   ... {{else}} ... {{else}} ... {{end}}
    let mut saw_unconditional_else = false;
    while p.at_left_delim_and(SyntaxKind::Else) {
        let c = p.checkpoint();
        let (else_clause_type, else_clause_range) = else_branch(p);
        if saw_unconditional_else {
            p.error(
                format!("{parent_context} must end immediately after first unconditional else branch"),
                else_clause_range,
            );
            p.wrap(c, SyntaxKind::Error);
        } else if else_clause_type == ElseBranchType::ElseIf && !permit_else_if {
            p.error(
                format!("{parent_context} does not support else-if branches"),
                else_clause_range,
            );
            p.wrap(c, SyntaxKind::Error)
        }

        saw_unconditional_else |= else_clause_type == ElseBranchType::Else;
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ElseBranchType {
    ElseIf,
    Else,
}

/// Return the type (`else if` or `else`) and text range of the else clause.
pub(crate) fn else_branch(p: &mut Parser) -> (ElseBranchType, TextRange) {
    let else_branch = p.start(SyntaxKind::ElseBranch);
    let (branch_type, range) = else_clause(p);
    action_list(p);
    else_branch.complete(p);
    (branch_type, range)
}

pub(crate) fn else_clause(p: &mut Parser) -> (ElseBranchType, TextRange) {
    let start = p.cur_start();

    let else_clause = p.start(SyntaxKind::ElseClause);
    left_delim(p);
    p.expect(SyntaxKind::Else);
    p.eat_whitespace();
    let branch_type = match p.cur() {
        SyntaxKind::RightDelim | SyntaxKind::TrimmedRightDelim => {
            p.eat();
            ElseBranchType::Else
        }
        SyntaxKind::If => {
            p.eat();
            expr(p, "after `else if`");
            p.eat_whitespace();
            right_delim(p);
            ElseBranchType::ElseIf
        }
        _ => {
            p.err_recover(
                format!(
                    "expected `if` keyword or right action delimiter after `else` keyword; found {}",
                    p.cur_name()
                ),
                LEFT_DELIMS,
            );
            ElseBranchType::Else
        }
    };
    else_clause.complete(p);
    (branch_type, TextRange::new(start, p.cur_start()))
}

pub(crate) fn end_clause(p: &mut Parser, parent_context: &str) {
    if !p.at_left_delim_and(SyntaxKind::End) {
        p.err_recover(format!("missing end clause for {parent_context}"), LEFT_DELIMS);
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

pub(crate) fn range_loop(p: &mut Parser) {
    let range_loop = p.start(SyntaxKind::RangeLoop);
    range_clause(p);
    action_list(p);
    else_branches(p, "range action", false);
    end_clause(p, "range loop");
    range_loop.complete(p);
}

pub(crate) fn range_clause(p: &mut Parser) {
    eprintln!("here");
    let range_clause = p.start(SyntaxKind::RangeClause);
    left_delim(p);
    p.eat_whitespace();
    p.expect(SyntaxKind::Range);
    if !p.eat_whitespace() {
        p.error_here(format!("expected space after `range` keyword; found {}", p.cur_name()))
    }

    // Iteration variables are tricky.

    let mut num_vars = 0;
    let mut saw_decl_or_assign_op = false;
    'parse_iter_vars: while p.at(SyntaxKind::Var) {
        match p.peek_non_space() {
            SyntaxKind::RightDelim | SyntaxKind::TrimmedRightDelim => {
                // {{range $x}}: $x is the expression to be iterated over,
                // so exit the loop and let expr() take care of it.
                break 'parse_iter_vars;
            }
            SyntaxKind::ColonEq => {
                saw_decl_or_assign_op = true;

                // {{range $x := expr}}: $x is the last iteration variable, so
                // eat `$x` and `:=` then exit the loop.
                p.assert(SyntaxKind::Var);
                num_vars += 1;
                p.eat_whitespace();
                p.assert(SyntaxKind::ColonEq);
                p.eat_whitespace();
                break 'parse_iter_vars;
            }
            SyntaxKind::Eq => {
                saw_decl_or_assign_op = false;

                // {{range $x = expr}}: similar to above.
                p.assert(SyntaxKind::Var);
                num_vars += 1;
                if !p.eat_whitespace() {
                    p.error_here("space required before `=` in assignment")
                }
                p.assert(SyntaxKind::Eq);
                p.eat_whitespace();
                break 'parse_iter_vars;
            }
            SyntaxKind::Comma => {
                // {{range $x, $y := expr}}: we are at `$x,` and still have more
                // iteration variables to parse.
                p.assert(SyntaxKind::Var);
                num_vars += 1;
                p.eat_whitespace();
                p.assert(SyntaxKind::Comma);
                p.eat_whitespace();
                continue 'parse_iter_vars;
            }
            SyntaxKind::Var => {
                // {{range $x $y := expr}}: we are at `$x`; this construct is
                // syntactically invalid but try to continue.
                p.assert(SyntaxKind::Var);
                num_vars += 1;
                p.eat_whitespace();
                p.error_here("expected comma separating variables in range");
                // don't eat the second variable; that's for the next iteration
                continue 'parse_iter_vars;
            }
            _ => {
                // Something unexpected; just let expr() take care of it.
                break 'parse_iter_vars;
            }
        }
    }

    if num_vars > 2 {
        p.error_here("too many iteration variables in range (max 2)");
    }
    if num_vars > 0 && !saw_decl_or_assign_op {
        p.error_here("expected `:=` or `=` between iteration variables and range expression");
    }

    expr(p, "in range action");
    eprintln!("expr returned");
    right_delim(p);
    range_clause.complete(p);
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
        p.err_and_eat(format!("expected left action delimiter; found {}", p.cur_name()));
    }
}

pub(crate) fn right_delim(p: &mut Parser) {
    while !p.at_eof() && !p.at(ACTION_DELIMS) {
        if p.eat_if(SyntaxKind::InvalidChar) {
            // lexer should already have emitted an error
        } else {
            p.err_and_eat(format!("unexpected {} in action", p.cur_name()))
        }
    }

    if !p.eat_if(RIGHT_DELIMS) {
        p.err_recover("expected right action delimiter", LEFT_DELIMS);
    }
}
