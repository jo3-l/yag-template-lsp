use crate::parser::expr::{arg, expr_pipeline};
use crate::parser::token_set::{TokenSet, ACTION_DELIMS, LEFT_DELIMS, RIGHT_DELIMS, STRING_LITERALS};
use crate::parser::{Parser, TokenPattern};
use crate::{SyntaxKind, TextRange};

impl Parser<'_> {
    fn at_left_delim_and(&mut self, pat: impl TokenPattern) -> bool {
        self.at(LEFT_DELIMS) && pat.matches(self.peek_ignore_space())
    }
}

fn action_list(p: &mut Parser) {
    const ACTION_LIST_TERMINATORS: TokenSet = TokenSet::new()
        .add(SyntaxKind::Else)
        .add(SyntaxKind::Catch)
        .add(SyntaxKind::End);

    let action_list = p.start(SyntaxKind::ActionList);
    // until EOF, `{{else`, `{{catch`, or `{{end`
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
        return p.err_and_eat(format!("expected left action delimiter; found {}", p.cur()));
    }
    match p.peek_ignore_space() {
        SyntaxKind::If => if_conditional(p),
        SyntaxKind::With => with_conditional(p),
        SyntaxKind::Range => range_loop(p),
        SyntaxKind::While => while_loop(p),
        SyntaxKind::Try => try_catch_action(p),
        SyntaxKind::Define => template_definition(p),
        SyntaxKind::Block => template_block(p),
        SyntaxKind::Template => template_invocation(p),
        SyntaxKind::Break => loop_break(p),
        SyntaxKind::Continue => loop_continue(p),
        SyntaxKind::Return => return_action(p),
        SyntaxKind::Else => p.wrap_err(else_clause, "unexpected {{else}}"),
        SyntaxKind::Catch => p.wrap_err(catch_clause, "unexpected {{catch}} outside of try-catch action"),
        SyntaxKind::End => p.wrap_err(end_clause, "unexpected {{end}}"),
        SyntaxKind::RightDelim | SyntaxKind::TrimmedRightDelim => {
            // peek_non_space saw a right delimiter, suggesting an empty action.
            // But since peek_non_space implicitly skips trivia, it also may be
            // the case that there's a comment in between:
            //    {{/* ... */}}
            // which is not an error -- we have to check.
            empty_or_comment_action(p)
        }
        _ => expr_action(p),
    }
}

fn if_conditional(p: &mut Parser) {
    let if_conditional = p.start(SyntaxKind::IfConditional);
    if_clause(p);
    action_list(p);
    else_branches(p, "if action", true);
    end_clause_or_recover(p, "if action");
    if_conditional.complete(p);
}

fn if_clause(p: &mut Parser) {
    let if_clause = p.start(SyntaxKind::IfClause);
    left_delim(p);
    p.eat_whitespace();
    p.expect(SyntaxKind::If);
    p.expect_whitespace("after `if` keyword");
    expr_pipeline(p, "after `if` keyword");
    p.eat_whitespace();
    right_delim_or_recover(p, "in if action");
    if_clause.complete(p);
}

fn with_conditional(p: &mut Parser) {
    let with_conditional = p.start(SyntaxKind::WithConditional);
    with_clause(p);
    action_list(p);
    else_branches(p, "with action", true);
    end_clause_or_recover(p, "with action");
    with_conditional.complete(p);
}

fn with_clause(p: &mut Parser) {
    let with_clause = p.start(SyntaxKind::WithClause);
    left_delim(p);
    p.eat_whitespace();
    p.expect(SyntaxKind::With);
    p.expect_whitespace("after `with` keyword");
    expr_pipeline(p, "after `with` keyword");
    p.eat_whitespace();
    right_delim_or_recover(p, "in with action");
    with_clause.complete(p);
}

fn else_branches(p: &mut Parser, parent_action_type: &str, permit_else_if: bool) {
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
                format!("{parent_action_type} must end immediately after first unconditional else branch"),
                else_clause_range,
            );
            p.wrap(c, SyntaxKind::Error);
        } else if else_clause_type == ElseBranchType::ElseIf && !permit_else_if {
            p.error(
                format!("{parent_action_type} does not support else-if branches"),
                else_clause_range,
            );
            p.wrap(c, SyntaxKind::Error)
        }

        saw_unconditional_else |= else_clause_type == ElseBranchType::Else;
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ElseBranchType {
    ElseIf,
    Else,
}

/// Return the type (`else if` or `else`) and text range of the else clause.
fn else_branch(p: &mut Parser) -> (ElseBranchType, TextRange) {
    let else_branch = p.start(SyntaxKind::ElseBranch);
    let (branch_type, range) = else_clause(p);
    action_list(p);
    else_branch.complete(p);
    (branch_type, range)
}

fn else_clause(p: &mut Parser) -> (ElseBranchType, TextRange) {
    let start = p.cur_start();

    let else_clause = p.start(SyntaxKind::ElseClause);
    left_delim(p);
    p.eat_whitespace();
    p.expect(SyntaxKind::Else);
    p.eat_whitespace();
    let branch_type = match p.cur() {
        SyntaxKind::RightDelim | SyntaxKind::TrimmedRightDelim => {
            right_delim(p);
            ElseBranchType::Else
        }
        SyntaxKind::If => {
            p.eat();
            p.eat_whitespace();
            expr_pipeline(p, "after `else if`");
            p.eat_whitespace();
            right_delim_or_recover(p, "in else-if clause");
            ElseBranchType::ElseIf
        }
        _ => {
            p.err_recover(
                format!(
                    "expected `if` keyword or right action delimiter after `else` keyword; found {}",
                    p.cur()
                ),
                LEFT_DELIMS,
            );
            ElseBranchType::Else
        }
    };
    else_clause.complete(p);
    (branch_type, TextRange::new(start, p.cur_start()))
}

fn end_clause_or_recover(p: &mut Parser, parent_action_type: &str) {
    if !p.at_left_delim_and(SyntaxKind::End) {
        p.err_recover(format!("missing end clause for {parent_action_type}"), LEFT_DELIMS);
        return;
    }

    end_clause(p);
}

fn end_clause(p: &mut Parser) {
    let end_clause = p.start(SyntaxKind::EndClause);
    left_delim(p);
    p.eat_whitespace();
    p.expect(SyntaxKind::End);
    p.eat_whitespace();
    right_delim_or_recover(p, "after `end` keyword");
    end_clause.complete(p);
}

fn range_loop(p: &mut Parser) {
    let range_loop = p.start(SyntaxKind::RangeLoop);
    range_clause(p);
    action_list(p);
    else_branches(p, "range loop", false);
    end_clause_or_recover(p, "range loop");
    range_loop.complete(p);
}

fn range_clause(p: &mut Parser) {
    let range_clause = p.start(SyntaxKind::RangeClause);
    left_delim(p);
    p.eat_whitespace();
    p.expect(SyntaxKind::Range);
    p.expect_whitespace("after `range` keyword");

    // Iteration variables are tricky.

    let mut num_vars = 0;
    let mut saw_decl_or_assign_op = false;
    'scan_iter_vars: while p.at(SyntaxKind::Var) {
        match p.peek_ignore_space() {
            // {{range $x}}: $x is the expression to be iterated over,
            // so exit the loop and let expr() take care of it.
            SyntaxKind::RightDelim | SyntaxKind::TrimmedRightDelim => {
                break 'scan_iter_vars;
            }

            // {{range $x := expr}}: $x is the last iteration variable, so
            // eat `$x` and `:=` then exit the loop.
            SyntaxKind::ColonEq => {
                saw_decl_or_assign_op = true;

                p.assert(SyntaxKind::Var);
                num_vars += 1;
                p.eat_whitespace();
                p.assert(SyntaxKind::ColonEq);
                p.eat_whitespace();
                break 'scan_iter_vars;
            }

            // {{range $x = expr}}: similar to above.
            SyntaxKind::Eq => {
                saw_decl_or_assign_op = false;

                p.assert(SyntaxKind::Var);
                num_vars += 1;
                p.expect_whitespace("before `=` in assignment");
                p.assert(SyntaxKind::Eq);
                p.eat_whitespace();
                break 'scan_iter_vars;
            }

            // {{range $x, $y := expr}}: we are at `$x,` and still have more
            // iteration variables to parse.
            SyntaxKind::Comma => {
                p.assert(SyntaxKind::Var);
                num_vars += 1;
                p.eat_whitespace();
                p.assert(SyntaxKind::Comma);
                p.eat_whitespace();
                continue 'scan_iter_vars;
            }

            // {{range $x $y := expr}}: we are at `$x`; this construct is
            // syntactically invalid but try to continue.
            SyntaxKind::Var => {
                p.assert(SyntaxKind::Var);
                num_vars += 1;
                p.eat_whitespace();
                p.error_here("expected comma separating variables in range");
                // don't eat the second variable; that's for the next iteration
                continue 'scan_iter_vars;
            }

            // Something unexpected; just let expr() take care of it.
            _ => {
                break 'scan_iter_vars;
            }
        }
    }

    if num_vars > 2 {
        p.error_here("too many iteration variables in range (max 2)");
    }
    if num_vars > 0 && !saw_decl_or_assign_op {
        p.error_here("expected `:=` or `=` between iteration variables and range expression");
    }

    expr_pipeline(p, "in range action");
    p.eat_whitespace();
    right_delim_or_recover(p, "in range action");
    range_clause.complete(p);
}

fn while_loop(p: &mut Parser) {
    let while_loop = p.start(SyntaxKind::WhileLoop);
    while_clause(p);
    action_list(p);
    else_branches(p, "while loop", false);
    end_clause_or_recover(p, "while loop");
    while_loop.complete(p);
}

fn while_clause(p: &mut Parser) {
    let while_clause = p.start(SyntaxKind::WhileClause);
    left_delim(p);
    p.eat_whitespace();
    p.expect(SyntaxKind::While);
    p.expect_whitespace("after `while` keyword");
    expr_pipeline(p, "after `while` keyword");
    p.eat_whitespace();
    right_delim_or_recover(p, "in `while` clause");
    while_clause.complete(p);
}

fn try_catch_action(p: &mut Parser) {
    let try_catch_action = p.start(SyntaxKind::TryCatchAction);
    try_clause(p);
    action_list(p);
    if p.at_left_delim_and(SyntaxKind::Catch) {
        catch_clause(p);
        action_list(p);
    } else {
        p.err_recover("missing {{catch}} for try-catch action", LEFT_DELIMS);
    }
    end_clause_or_recover(p, "try-catch action");
    try_catch_action.complete(p);
}

fn try_clause(p: &mut Parser) {
    let try_clause = p.start(SyntaxKind::TryClause);
    left_delim(p);
    p.eat_whitespace();
    p.expect(SyntaxKind::Try);
    p.eat_whitespace();
    right_delim_or_recover(p, "after `try` keyword");
    try_clause.complete(p);
}

fn catch_clause(p: &mut Parser) {
    let catch_clause = p.start(SyntaxKind::CatchClause);
    left_delim(p);
    p.eat_whitespace();
    p.expect(SyntaxKind::Catch);
    p.eat_whitespace();
    right_delim_or_recover(p, "after `catch` keyword");
    catch_clause.complete(p);
}

// FIXME: Need to reject template definitions not at top level:
fn template_definition(p: &mut Parser) {
    let template_definition = p.start(SyntaxKind::TemplateDefinition);
    define_clause(p);
    action_list(p);
    end_clause_or_recover(p, "template definition");
    template_definition.complete(p);
}

fn define_clause(p: &mut Parser) {
    let define_clause = p.start(SyntaxKind::DefineClause);
    left_delim(p);
    p.eat_whitespace();
    p.expect(SyntaxKind::Define);
    p.expect_whitespace("after `define` keyword");

    if !p.eat_if(STRING_LITERALS) {
        p.err_recover(
            format!("expected name of template after `define` keyword; found {}", p.cur()),
            ACTION_DELIMS,
        );
    }
    p.eat_whitespace();
    right_delim_or_recover(p, "in `define` clause");
    define_clause.complete(p);
}

fn template_block(p: &mut Parser) {
    let template_block = p.start(SyntaxKind::TemplateBlock);
    block_clause(p);
    action_list(p);
    end_clause_or_recover(p, "template block");
    template_block.complete(p);
}

fn block_clause(p: &mut Parser) {
    let block_clause = p.start(SyntaxKind::BlockClause);
    left_delim(p);
    p.expect(SyntaxKind::Block);
    p.expect_whitespace("after `block` keyword");

    if !p.eat_if(STRING_LITERALS) {
        p.err_recover(
            format!("expected name of template after `block` keyword; found {}", p.cur()),
            ACTION_DELIMS,
        );
    }

    // Accept an optional expression denoting the context data to send.
    if !p.at_ignore_space(RIGHT_DELIMS) {
        p.expect_whitespace("separating template name and context data");
        expr_pipeline(p, "for template block");
    }

    p.eat_whitespace();
    right_delim_or_recover(p, "in `block` clause");
    block_clause.complete(p);
}

fn template_invocation(p: &mut Parser) {
    let template_invocation = p.start(SyntaxKind::TemplateInvocation);
    left_delim(p);
    p.expect(SyntaxKind::Template);
    p.expect_whitespace("after `template` keyword");

    match p.cur() {
        SyntaxKind::InterpretedString | SyntaxKind::RawString => p.eat(),
        SyntaxKind::RightDelim | SyntaxKind::TrimmedRightDelim => {
            p.error_here("expected name of template to invoke after `template` keyword")
        }
        _ => {
            // Perhaps something like `{{template $x}}`; though this construct
            // is erroneous (`template` only works with constant string literal
            // names), try to parse it and issue an error.
            p.wrap_err(arg, "template invocations only accept constant string literal names");
        }
    }
    // Accept an optional expression denoting the context data to send.
    if !p.at_ignore_space(RIGHT_DELIMS) {
        p.expect_whitespace("separating template name and context data");
        expr_pipeline(p, "to pass to template");
    }
    p.eat_whitespace();
    right_delim_or_recover(p, "in `template` invocation");
    template_invocation.complete(p);
}

fn loop_break(p: &mut Parser) {
    let loop_break = p.start(SyntaxKind::LoopBreak);
    left_delim(p);
    p.eat_whitespace();
    p.expect(SyntaxKind::Break);
    p.eat_whitespace();
    right_delim_or_recover(p, "after `break` keyword");
    loop_break.complete(p);
}

fn loop_continue(p: &mut Parser) {
    let loop_continue = p.start(SyntaxKind::LoopContinue);
    left_delim(p);
    p.eat_whitespace();
    p.expect(SyntaxKind::Continue);
    p.eat_whitespace();
    right_delim_or_recover(p, "after `continue` keyword");
    loop_continue.complete(p);
}

fn return_action(p: &mut Parser) {
    let return_action = p.start(SyntaxKind::ReturnAction);
    left_delim(p);
    p.eat_whitespace();
    p.expect(SyntaxKind::Return);
    if !p.at_ignore_space(RIGHT_DELIMS) {
        p.expect_whitespace("separating `return` keyword and expression");
        expr_pipeline(p, "to return");
    }
    p.eat_whitespace();
    right_delim_or_recover(p, "in `return` action");
    return_action.complete(p);
}

// FIXME: Need to validate that comments only appear in valid positions;
// {{ /* comment */ }} with the space is supposed to be an error and in general
// the only valid location for a comment is when it appears exactly
// like {{/* comment */}}. (Currently we accept comments nearly everywhere as a
// consequence of implicitly skipping trivia.)
fn empty_or_comment_action(p: &mut Parser) {
    // NOTE: The implementation is complicated by the fact that most Parser
    // methods implicitly skip comments (trivia), but since we actually care
    // about comments we must restrict ourselves to using
    // Parser::only_eat_cur_token.

    let c = p.checkpoint();
    let pos = p.cur_start();

    assert!(p.at(LEFT_DELIMS));
    p.only_eat_cur_token();
    if p.at_ignore_space(SyntaxKind::Comment) {
        p.only_eat_cur_token();
        right_delim_or_recover(p, "after first comment in action");
        p.wrap(c, SyntaxKind::CommentAction);
    } else {
        // no immediate comment means we are looking at an action that consists
        // of a comment in an invalid position (e.g., `{{ /* comment */ }}` --
        // note the spacing) or an empty action.
        p.eat_whitespace();
        right_delim(p);
        p.wrap(c, SyntaxKind::Error);

        // FIXME: This error is misleading if the action contains a comment with
        // leading whitespace, such as `{{ /* comment */ }}`.
        p.error("unexpected empty action", TextRange::new(pos, p.cur_start()));
    }
}

fn expr_action(p: &mut Parser) {
    let expr_action = p.start(SyntaxKind::ExprAction);
    left_delim(p);
    p.eat_whitespace();
    expr_pipeline(p, "after `{{`");
    p.eat_whitespace();
    right_delim_or_recover(p, "in action");
    expr_action.complete(p);
}

fn left_delim(p: &mut Parser) {
    if !p.eat_if(LEFT_DELIMS) {
        p.err_and_eat(format!("expected left action delimiter; found {}", p.cur()));
    }
}

fn right_delim_or_recover(p: &mut Parser, context: &str) {
    while !p.at_eof() && !p.at(ACTION_DELIMS) {
        if p.eat_if(SyntaxKind::InvalidCharInAction) {
            // lexer should already have emitted an error; no need for another
        } else {
            p.err_and_eat(format!("unexpected {} {context}", p.cur()))
        }
    }

    right_delim(p);
}

fn right_delim(p: &mut Parser) {
    if !p.eat_if(RIGHT_DELIMS) {
        p.err_recover("expected right action delimiter", LEFT_DELIMS);
    }
}
