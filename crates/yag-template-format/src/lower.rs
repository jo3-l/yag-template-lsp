//! AST-to-document lowering for the formatter's conservative early stages.
//!
//! Ordinary delimiter padding and parsed block indentation are intentionally
//! separate from the later expression-layout stage. Trim, comments, and
//! multi-line actions consequently remain source-verbatim here.

use std::cell::RefCell;
use std::ops::Range;

use yag_template_syntax::ast::{
    Action, ActionList, ActionOrText, AstNode, AstToken, CommentAction, Expr, ExprAction, Root, TemplateBlock,
    TemplateDefinition, TryCatchAction,
};
use yag_template_syntax::{SyntaxKind, SyntaxNode};

use crate::doc::Doc;
use crate::region::{LayoutPolicy, LinePlan};
use crate::{DanglingValuePolicy, DelimiterPadding, FormatDiagnostic, FormatDiagnosticKind, FormatOptions, LayoutKind};

pub(super) fn lower(
    root: &SyntaxNode,
    source: &str,
    options: &FormatOptions,
    plan: &LinePlan,
) -> (Doc, Vec<FormatDiagnostic>) {
    let Some(root) = Root::cast(root.clone()) else {
        return (Doc::verbatim(source), Vec::new());
    };
    let lowerer = Lowerer {
        source,
        options,
        plan,
        diagnostics: RefCell::new(Vec::new()),
    };
    let doc = lowerer.root(root);
    (doc, lowerer.diagnostics.into_inner())
}

struct Lowerer<'a> {
    source: &'a str,
    options: &'a FormatOptions,
    plan: &'a LinePlan,
    diagnostics: RefCell<Vec<FormatDiagnostic>>,
}

impl Lowerer<'_> {
    fn root(&self, root: Root) -> Doc {
        self.sequence(root.actions_with_text(), None)
    }

    fn action_list(&self, action_list: ActionList, normalize_margins: bool) -> Doc {
        let range = source_range(&action_list);
        self.sequence(action_list.actions_with_text(), normalize_margins.then_some(range.end))
    }

    fn sequence(&self, elements: impl Iterator<Item = ActionOrText>, sequence_end: Option<usize>) -> Doc {
        Doc::concat(elements.map(|element| match element {
            ActionOrText::Action(action) => self.action(action),
            ActionOrText::Text(text) => self.text(text, sequence_end),
        }))
    }

    fn action(&self, action: Action) -> Doc {
        match action {
            Action::TemplateDefinition(action) => self.template_definition(action),
            Action::TemplateBlock(action) => self.template_block(action),
            Action::If(action) => self.if_action(action),
            Action::With(action) => self.with_action(action),
            Action::Range(action) => self.range_action(action),
            Action::While(action) => self.while_action(action),
            Action::TryCatch(action) => self.try_catch_action(action),
            Action::Comment(action) => self.comment(action),
            Action::TemplateInvocation(action) => self.delimited_expr(action.clone(), action.context_data()),
            Action::Return(action) => self.delimited_expr(action.clone(), action.expr()),
            Action::ExprAction(action) => self.expr_action(action),
            action @ (Action::Break(_) | Action::Continue(_)) => self.delimited(action),
        }
    }

    fn expr_action(&self, action: ExprAction) -> Doc {
        self.delimited_expr(action.clone(), action.expr())
    }

    fn comment(&self, action: CommentAction) -> Doc {
        // Go template comments require the comment marker to remain adjacent
        // to the opening delimiter. Keep comments exact under both padding
        // modes rather than accidentally producing a non-portable spelling.
        self.verbatim(&action)
    }

    fn template_definition(&self, action: TemplateDefinition) -> Doc {
        self.compound(
            &action,
            [
                action.clause().map(|clause| self.delimited(clause)),
                action.template_body().map(|body| self.body(body)),
                action.end_clause().map(|clause| self.delimited(clause)),
            ],
        )
    }

    fn template_block(&self, action: TemplateBlock) -> Doc {
        self.compound(
            &action,
            [
                action
                    .clause()
                    .map(|clause| self.delimited_expr(clause.clone(), clause.context_data())),
                action.template_body().map(|body| self.body(body)),
                action.end_clause().map(|clause| self.delimited(clause)),
            ],
        )
    }

    fn if_action(&self, action: yag_template_syntax::ast::IfAction) -> Doc {
        let mut parts = vec![
            action
                .clause()
                .map(|clause| self.delimited_expr(clause.clone(), clause.condition())),
            action.body().map(|body| self.body(body)),
        ];
        for branch in action.else_branches() {
            parts.push(
                branch
                    .clause()
                    .map(|clause| self.delimited_expr(clause.clone(), clause.condition())),
            );
            parts.push(branch.body().map(|body| self.body(body)));
        }
        parts.push(action.end_clause().map(|clause| self.delimited(clause)));
        self.compound(&action, parts)
    }

    fn with_action(&self, action: yag_template_syntax::ast::WithAction) -> Doc {
        let mut parts = vec![
            action
                .clause()
                .map(|clause| self.delimited_expr(clause.clone(), clause.condition())),
            action.body().map(|body| self.body(body)),
        ];
        for branch in action.else_branches() {
            parts.push(
                branch
                    .clause()
                    .map(|clause| self.delimited_expr(clause.clone(), clause.condition())),
            );
            parts.push(branch.body().map(|body| self.body(body)));
        }
        parts.push(action.end_clause().map(|clause| self.delimited(clause)));
        self.compound(&action, parts)
    }

    fn range_action(&self, action: yag_template_syntax::ast::RangeLoop) -> Doc {
        self.compound(
            &action,
            [
                action
                    .clause()
                    .map(|clause| self.delimited_expr(clause.clone(), clause.expr())),
                action.body().map(|body| self.body(body)),
                action.else_branch().and_then(|branch| {
                    branch
                        .clause()
                        .map(|clause| self.delimited_expr(clause.clone(), clause.condition()))
                }),
                action
                    .else_branch()
                    .and_then(|branch| branch.body().map(|body| self.body(body))),
                action.end_clause().map(|clause| self.delimited(clause)),
            ],
        )
    }

    fn while_action(&self, action: yag_template_syntax::ast::WhileLoop) -> Doc {
        self.compound(
            &action,
            [
                action
                    .clause()
                    .map(|clause| self.delimited_expr(clause.clone(), clause.condition())),
                action.body().map(|body| self.body(body)),
                action.else_branch().and_then(|branch| {
                    branch
                        .clause()
                        .map(|clause| self.delimited_expr(clause.clone(), clause.condition()))
                }),
                action
                    .else_branch()
                    .and_then(|branch| branch.body().map(|body| self.body(body))),
                action.end_clause().map(|clause| self.delimited(clause)),
            ],
        )
    }

    fn try_catch_action(&self, action: TryCatchAction) -> Doc {
        self.compound(
            &action,
            [
                action.try_clause().map(|clause| self.delimited(clause)),
                action.try_body().map(|body| self.body(body)),
                action.catch_clause().map(|clause| self.delimited(clause)),
                action.catch_body().map(|body| self.body(body)),
                action.end_clause().map(|clause| self.delimited(clause)),
            ],
        )
    }

    /// Lower a typed compound body. A final source newline belongs to the
    /// surrounding block boundary, so it is emitted after the nested body and
    /// aligns the following `else`, `catch`, or `end` with the header.
    fn body(&self, body: ActionList) -> Doc {
        let range = source_range(&body);
        let has_terminal_newline = ends_with_line_break_after_margin(&self.source[range]);
        let content = self.action_list(body, true);
        let content = Doc::nest(self.options.indent, content);
        if has_terminal_newline {
            Doc::concat([content, Doc::Line])
        } else {
            content
        }
    }

    /// Preserve literal content while letting block layout own physical-line
    /// margins. This never strips whitespace at an action/text boundary on a
    /// shared source line; it only removes whitespace before a source newline
    /// or after one.
    fn text(&self, text: yag_template_syntax::ast::Text, sequence_end: Option<usize>) -> Doc {
        let range = text.text_range();
        let start = byte_offset(range.start());
        let end = byte_offset(range.end());
        let Some(sequence_end) = sequence_end else {
            return Doc::verbatim(text.get());
        };

        let mut at_line_start = start == 0 || self.source.as_bytes()[start - 1] == b'\n';
        let mut parts = Vec::new();
        let mut consumed = 0;
        let terminal_newline_end = (end == sequence_end)
            .then(|| final_line_break_end(text.get()))
            .flatten();
        for segment in text.get().split_inclusive('\n') {
            consumed += segment.len();
            let has_newline = segment.ends_with('\n');
            let mut content = segment.strip_suffix('\n').unwrap_or(segment);
            if at_line_start {
                content = content.trim_start_matches(char::is_whitespace);
            }
            if has_newline {
                content = content.trim_end_matches(char::is_whitespace);
            }
            if !content.is_empty() {
                parts.push(Doc::verbatim(content));
            }
            if has_newline {
                let terminal_newline = terminal_newline_end == Some(consumed);
                if !terminal_newline {
                    parts.push(Doc::Line);
                }
                at_line_start = true;
            } else {
                at_line_start = false;
            }
        }
        Doc::concat(parts)
    }

    fn compound<N: AstNode>(&self, action: &N, parts: impl IntoIterator<Item = Option<Doc>>) -> Doc {
        let parts = parts.into_iter().collect::<Option<Vec<_>>>();
        parts.map_or_else(|| self.verbatim(action), Doc::concat)
    }

    fn delimited<N: AstNode>(&self, node: N) -> Doc {
        let range = source_range(&node);
        let Some((left_end, right_start)) = self.delimiter_body_range(&node) else {
            return self.verbatim(&node);
        };
        self.delimited_body(
            node,
            range,
            left_end,
            right_start,
            Doc::verbatim(self.source[left_end..right_start].trim()),
        )
    }

    fn delimited_expr<N: AstNode>(&self, node: N, expr: Option<Expr>) -> Doc {
        let Some(expr) = expr else {
            return self.delimited(node);
        };
        let range = source_range(&node);
        let Some((left_end, right_start)) = self.delimiter_body_range(&node) else {
            return self.verbatim(&node);
        };
        let expr_range = source_range(&expr);
        let prefix = self.source[left_end..expr_range.start].trim();
        let suffix = self.source[expr_range.end..right_start].trim();
        let expression = self.expr(expr);
        let mut tail = vec![Doc::SoftLine, expression];
        if !suffix.is_empty() {
            tail.extend([Doc::SoftLine, Doc::text(suffix)]);
        }
        let body = if prefix.is_empty() {
            tail.remove(0);
            Doc::group(Doc::concat(tail))
        } else {
            Doc::group(Doc::concat([
                Doc::text(prefix),
                Doc::nest(self.options.continuation_indent, Doc::concat(tail)),
            ]))
        };
        self.delimited_body(node, range, left_end, right_start, body)
    }

    fn delimiter_body_range<N: AstNode>(&self, node: &N) -> Option<(usize, usize)> {
        let left = node.syntax().first_token()?;
        let right = node.syntax().last_token()?;
        if !matches!(left.kind(), SyntaxKind::LeftDelim | SyntaxKind::TrimmedLeftDelim)
            || !matches!(right.kind(), SyntaxKind::RightDelim | SyntaxKind::TrimmedRightDelim)
        {
            return None;
        }
        Some((
            byte_offset(left.text_range().end()),
            byte_offset(right.text_range().start()),
        ))
    }

    fn delimited_body<N: AstNode>(
        &self,
        node: N,
        range: Range<usize>,
        left_end: usize,
        right_start: usize,
        body: Doc,
    ) -> Doc {
        let source = &self.source[range.clone()];
        let left = &self.source[range.start..left_end];
        let right = &self.source[right_start..range.end];
        if self.plan.range_contains_protected_line(range.clone()) && source.contains('\n') {
            return Doc::verbatim(source);
        }

        // Preserve trim spellings exactly. In particular, `{{- .Value -}}`
        // has whitespace required by the trim-marker grammar.
        if left.contains('-') || right.contains('-') || source.contains('\n') {
            return Doc::verbatim(source);
        }
        let padding = match self.options.delimiter_padding {
            DelimiterPadding::None => "",
            DelimiterPadding::Spaces => " ",
        };
        let doc = Doc::concat([
            Doc::text(left),
            Doc::text(padding),
            body,
            Doc::text(padding),
            Doc::text(right),
        ]);
        if self.plan.policy_at_offset(range.start) == LayoutPolicy::Protected {
            doc.flatten().unwrap_or_else(|| self.verbatim(&node))
        } else {
            Doc::group(doc)
        }
    }

    fn expr(&self, expr: Expr) -> Doc {
        match expr {
            Expr::FuncCall(call) => call.func_name().map_or_else(
                || self.verbatim(&call),
                |name| self.function_call(name.get(), call.args().map(|arg| self.expr(arg))),
            ),
            Expr::ExprCall(call) => call.callee().map_or_else(
                || self.verbatim(&call),
                |callee| self.call(self.expr(callee), call.args().map(|arg| self.expr(arg))),
            ),
            Expr::Parenthesized(parenthesized) => parenthesized.inner_expr().map_or_else(
                || self.verbatim(&parenthesized),
                |inner| Doc::concat([Doc::text("("), self.expr(inner), Doc::text(")")]),
            ),
            Expr::Pipeline(pipeline) => pipeline.init_expr().map_or_else(
                || self.verbatim(&pipeline),
                |init| {
                    let mut parts = vec![self.expr(init)];
                    for stage in pipeline.stages() {
                        let Some(call) = stage.call_expr() else {
                            return self.verbatim(&pipeline);
                        };
                        parts.extend([Doc::SoftLine, Doc::text("| "), self.expr(call)]);
                    }
                    Doc::group(Doc::concat([
                        parts.remove(0),
                        Doc::nest(self.options.continuation_indent, Doc::concat(parts)),
                    ]))
                },
            ),
            Expr::ContextAccess(access) => self.verbatim(&access),
            Expr::ContextFieldChain(chain) => Doc::text(
                chain
                    .fields()
                    .map(|field| field.syntax().text().to_owned())
                    .collect::<String>(),
            ),
            Expr::ExprFieldChain(chain) => chain.base_expr().map_or_else(
                || self.verbatim(&chain),
                |base| {
                    Doc::concat([
                        self.expr(base),
                        Doc::text(
                            chain
                                .fields()
                                .map(|field| field.syntax().text().to_owned())
                                .collect::<String>(),
                        ),
                    ])
                },
            ),
            Expr::VarAccess(access) => access
                .var()
                .map_or_else(|| self.verbatim(&access), |var| Doc::text(var.name())),
            Expr::VarDecl(decl) => self.assignment(
                &decl,
                ":=",
                decl.var().map(|var| var.name().to_owned()),
                decl.initializer(),
            ),
            Expr::VarAssign(assign) => self.assignment(
                &assign,
                "=",
                assign.var().map(|var| var.name().to_owned()),
                assign.assign_expr(),
            ),
            Expr::Literal(literal) => self.verbatim(&literal),
        }
    }

    fn function_call(&self, name: &str, args: impl Iterator<Item = Doc>) -> Doc {
        let args = args.collect::<Vec<_>>();
        match self.options.function_layouts.by_name.get(name) {
            Some(LayoutKind::KeyValuePairs { dangling_value }) if args.len() % 2 == 0 => {
                self.key_value_call(Doc::text(name), args)
            }
            Some(LayoutKind::KeyValuePairs { dangling_value }) => {
                self.diagnostics.borrow_mut().push(FormatDiagnostic {
                    kind: FormatDiagnosticKind::OddKeyValueArgumentCount,
                    message: format!("key-value function `{name}` received an odd number of arguments"),
                });
                match dangling_value {
                    DanglingValuePolicy::PreserveCallLayout | DanglingValuePolicy::Error => {
                        self.call(Doc::text(name), args)
                    }
                }
            }
            Some(LayoutKind::Call) | None => self.call(Doc::text(name), args),
        }
    }

    fn call(&self, callee: Doc, args: impl IntoIterator<Item = Doc>) -> Doc {
        let args = args.into_iter().collect::<Vec<_>>();
        if args.is_empty() {
            return callee;
        }
        Doc::group(Doc::concat([
            callee,
            Doc::nest(
                self.options.continuation_indent,
                Doc::concat(args.into_iter().flat_map(|arg| [Doc::SoftLine, arg])),
            ),
        ]))
    }

    fn key_value_call(&self, callee: Doc, args: Vec<Doc>) -> Doc {
        let rows = args
            .chunks_exact(2)
            .flat_map(|pair| [Doc::SoftLine, pair[0].clone(), Doc::text(" "), pair[1].clone()]);
        Doc::group(Doc::concat([
            callee,
            Doc::nest(self.options.continuation_indent, Doc::concat(rows)),
        ]))
    }

    fn assignment<N: AstNode>(&self, node: &N, operator: &str, variable: Option<String>, value: Option<Expr>) -> Doc {
        match (variable, value) {
            (Some(variable), Some(value)) => Doc::group(Doc::concat([
                Doc::text(variable),
                Doc::text(" "),
                Doc::text(operator),
                Doc::nest(
                    self.options.continuation_indent,
                    Doc::concat([Doc::SoftLine, self.expr(value)]),
                ),
            ])),
            _ => self.verbatim(node),
        }
    }

    fn verbatim<N: AstNode>(&self, node: &N) -> Doc {
        Doc::verbatim(&self.source[source_range(node)])
    }
}

fn source_range(node: &impl AstNode) -> Range<usize> {
    let range = node.text_range();
    byte_offset(range.start())..byte_offset(range.end())
}

fn byte_offset(position: impl Into<u32>) -> usize {
    position.into() as usize
}

fn ends_with_line_break_after_margin(text: &str) -> bool {
    final_line_break_end(text).is_some()
}

/// The byte offset immediately after a final newline followed only by physical
/// line-margin whitespace.
fn final_line_break_end(text: &str) -> Option<usize> {
    let without_margin = text.trim_end_matches(|character: char| character != '\n' && character.is_whitespace());
    without_margin.ends_with('\n').then_some(without_margin.len())
}
