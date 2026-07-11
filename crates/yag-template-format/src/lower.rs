//! AST-to-document lowering for the formatter's conservative early stages.
//!
//! Ordinary delimiter padding and parsed block indentation are intentionally
//! separate from the later expression-layout stage. Trim, comments, and
//! multi-line actions consequently remain source-verbatim here.

use std::ops::Range;

use yag_template_syntax::ast::{
    Action, ActionList, ActionOrText, AstNode, AstToken, CommentAction, Root, TemplateBlock, TemplateDefinition,
    TryCatchAction,
};
use yag_template_syntax::{SyntaxKind, SyntaxNode};

use crate::doc::Doc;
use crate::region::LinePlan;
use crate::{DelimiterPadding, FormatOptions};

pub(super) fn lower(root: &SyntaxNode, source: &str, options: &FormatOptions, plan: &LinePlan) -> Doc {
    let Some(root) = Root::cast(root.clone()) else {
        return Doc::verbatim(source);
    };
    Lowerer { source, options, plan }.root(root)
}

struct Lowerer<'a> {
    source: &'a str,
    options: &'a FormatOptions,
    plan: &'a LinePlan,
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
            action @ (Action::TemplateInvocation(_)
            | Action::Return(_)
            | Action::Break(_)
            | Action::Continue(_)
            | Action::ExprAction(_)) => self.delimited(action),
        }
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
                action.clause().map(|clause| self.delimited(clause)),
                action.template_body().map(|body| self.body(body)),
                action.end_clause().map(|clause| self.delimited(clause)),
            ],
        )
    }

    fn if_action(&self, action: yag_template_syntax::ast::IfAction) -> Doc {
        let mut parts = vec![
            action.clause().map(|clause| self.delimited(clause)),
            action.body().map(|body| self.body(body)),
        ];
        for branch in action.else_branches() {
            parts.push(branch.clause().map(|clause| self.delimited(clause)));
            parts.push(branch.body().map(|body| self.body(body)));
        }
        parts.push(action.end_clause().map(|clause| self.delimited(clause)));
        self.compound(&action, parts)
    }

    fn with_action(&self, action: yag_template_syntax::ast::WithAction) -> Doc {
        let mut parts = vec![
            action.clause().map(|clause| self.delimited(clause)),
            action.body().map(|body| self.body(body)),
        ];
        for branch in action.else_branches() {
            parts.push(branch.clause().map(|clause| self.delimited(clause)));
            parts.push(branch.body().map(|body| self.body(body)));
        }
        parts.push(action.end_clause().map(|clause| self.delimited(clause)));
        self.compound(&action, parts)
    }

    fn range_action(&self, action: yag_template_syntax::ast::RangeLoop) -> Doc {
        self.compound(
            &action,
            [
                action.clause().map(|clause| self.delimited(clause)),
                action.body().map(|body| self.body(body)),
                action
                    .else_branch()
                    .and_then(|branch| branch.clause().map(|clause| self.delimited(clause))),
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
                action.clause().map(|clause| self.delimited(clause)),
                action.body().map(|body| self.body(body)),
                action
                    .else_branch()
                    .and_then(|branch| branch.clause().map(|clause| self.delimited(clause))),
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
        let source = &self.source[range.clone()];
        if self.plan.range_contains_protected_line(range) && source.contains('\n') {
            return Doc::verbatim(source);
        }

        let Some(left) = node.syntax().first_token() else {
            return Doc::verbatim(source);
        };
        let Some(right) = node.syntax().last_token() else {
            return Doc::verbatim(source);
        };
        if !matches!(left.kind(), SyntaxKind::LeftDelim | SyntaxKind::TrimmedLeftDelim)
            || !matches!(right.kind(), SyntaxKind::RightDelim | SyntaxKind::TrimmedRightDelim)
        {
            return Doc::verbatim(source);
        }

        // Preserve trim spellings exactly. In particular, `{{- .Value -}}`
        // has whitespace required by the trim-marker grammar.
        if matches!(left.kind(), SyntaxKind::TrimmedLeftDelim)
            || matches!(right.kind(), SyntaxKind::TrimmedRightDelim)
            || source.contains('\n')
        {
            return Doc::verbatim(source);
        }

        let left_end = byte_offset(left.text_range().end());
        let right_start = byte_offset(right.text_range().start());
        let body = self.source[left_end..right_start].trim();
        let padding = match self.options.delimiter_padding {
            DelimiterPadding::None => "",
            DelimiterPadding::Spaces => " ",
        };
        Doc::concat([
            Doc::text(left.text()),
            Doc::text(padding),
            Doc::verbatim(body),
            Doc::text(padding),
            Doc::text(right.text()),
        ])
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
