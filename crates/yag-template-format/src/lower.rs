//! AST-to-document lowering and source-layout ownership.
//!
//! `Formatter` owns the formatting inputs and diagnostics, but never stores
//! partially built documents. Typed rules return complete fragments, allowing
//! a failed rule to discard its work and preserve the original action source.

use std::ops::Range;

use yag_template_syntax::SyntaxNode;
use yag_template_syntax::ast::{ActionList, ActionOrText, AstNode, AstToken, LeftDelim, RightDelim, Root};

use crate::classification::{LayoutPolicy, LinePlan};
use crate::doc::{Doc, concat, group, line, nest, text};
use crate::{DelimiterPadding, FormatDiagnostic, FormatOptions, LayoutKind};

pub(super) fn lower(
    root: &SyntaxNode,
    source: &str,
    options: &FormatOptions,
    plan: &LinePlan,
) -> (Doc, Vec<FormatDiagnostic>) {
    let Some(root) = Root::cast(root.clone()) else {
        return (text(source), Vec::new());
    };
    let mut formatter = Formatter::new(source, options, plan);
    let doc = formatter.sequence(root.actions_with_text(), None);
    (doc, formatter.diagnostics)
}

/// Stateful services shared by typed rules.
///
/// The formatter owns source/configuration access and diagnostics only.
/// Document fragments remain return values, so no failed rule can leak partial
/// output into its caller.
pub(crate) struct Formatter<'a> {
    source: &'a str,
    options: &'a FormatOptions,
    plan: &'a LinePlan,
    diagnostics: Vec<FormatDiagnostic>,
}

impl<'a> Formatter<'a> {
    fn new(source: &'a str, options: &'a FormatOptions, plan: &'a LinePlan) -> Self {
        Self {
            source,
            options,
            plan,
            diagnostics: Vec::new(),
        }
    }

    pub(crate) fn report(&mut self, diagnostic: FormatDiagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub(crate) fn function_layout(&self, name: &str) -> Option<LayoutKind> {
        self.options.function_layouts.by_name.get(name).copied()
    }

    pub(crate) fn continuation(&self, doc: Doc) -> Doc {
        nest(self.options.continuation_indent, doc)
    }

    /// Wrap an explicit pair of typed action delimiters around `body`.
    pub(crate) fn delimited(&self, (left, right): (LeftDelim, RightDelim), body: Doc) -> Option<Doc> {
        let left_range = left.text_range();
        let right_range = right.text_range();
        let range = byte_offset(left_range.start())..byte_offset(right_range.end());
        let left_end = byte_offset(left_range.end());
        let right_start = byte_offset(right_range.start());
        let original = &self.source[range.clone()];
        if self.plan.range_contains_protected_line(range.clone()) && original.contains('\n') {
            return Some(text(original));
        }

        // Preserve a multi-line action's interior layout, but still normalize
        // horizontal padding immediately inside ordinary delimiters. This
        // keeps a source-preserved body from opting out of delimiter options.
        if original.contains('\n') {
            let body = format_multiline_delimited_body(
                &self.source[left_end..right_start],
                left.has_trim_marker(),
                right.has_trim_marker(),
                self.options.delimiter_padding,
            );
            let mut action = String::with_capacity(original.len());
            action.push_str(&self.source[range.start..left_end]);
            action.push_str(&body);
            action.push_str(&self.source[right_start..range.end]);
            return Some(text(action));
        }

        let padding = match self.options.delimiter_padding {
            DelimiterPadding::None => "",
            DelimiterPadding::Spaces => " ",
        };
        let left_padding = if left.has_trim_marker() { "" } else { padding };
        let right_padding = if right.has_trim_marker() { "" } else { padding };
        let doc = concat([
            text(&self.source[range.start..left_end]),
            // A trim marker's token includes its grammar-required space, so
            // delimiter padding applies only to its ordinary counterpart.
            text(left_padding),
            body,
            text(right_padding),
            text(&self.source[right_start..range.end]),
        ]);
        if self.policy_at_offset(range.start) == LayoutPolicy::Protected {
            doc.flatten()
        } else {
            Some(group(doc))
        }
    }

    /// Format a typed compound body. A final source newline belongs to the
    /// surrounding block boundary, so it aligns the following `else`, `catch`,
    /// or `end` with the header.
    pub(crate) fn body(&mut self, body: ActionList) -> Doc {
        let range = source_range(&body);
        let sequence_end = range.end;
        let has_terminal_newline = ends_with_line_break_after_margin(&self.source[range]);
        let content = nest(
            self.options.indent,
            self.sequence(body.actions_with_text(), Some(sequence_end)),
        );
        if has_terminal_newline {
            concat([content, line()])
        } else {
            content
        }
    }

    /// Format a direct root/body sequence. It owns structural action separation
    /// and literal-text physical margins, which are source relationships rather
    /// than properties of a single syntax node.
    fn sequence(&mut self, elements: impl Iterator<Item = ActionOrText>, sequence_end: Option<usize>) -> Doc {
        let elements = elements.collect::<Vec<_>>();
        let mut documents = Vec::new();
        for (index, element) in elements.iter().cloned().enumerate() {
            match element {
                ActionOrText::Action(action) => {
                    if index > 0
                        && matches!(elements[index - 1], ActionOrText::Action(_))
                        && self.policy_at_offset(source_range(&action).start) == LayoutPolicy::Flexible
                    {
                        documents.push(line());
                    }
                    documents.push(self.action(action));
                }
                ActionOrText::Text(text) => {
                    let separates_flexible_actions = !text.get().contains('\n')
                        && text.get().chars().all(char::is_whitespace)
                        && index > 0
                        && index + 1 < elements.len()
                        && matches!(elements[index - 1], ActionOrText::Action(_))
                        && matches!(elements[index + 1], ActionOrText::Action(_))
                        && self.policy_at_offset(byte_offset(text.text_range().start())) == LayoutPolicy::Flexible;
                    if separates_flexible_actions {
                        documents.push(line());
                    } else {
                        documents.push(self.literal_text(text, sequence_end));
                    }
                }
            }
        }
        concat(documents)
    }

    /// Preserve literal content while letting block layout own physical-line
    /// margins. This never strips whitespace at an action/text boundary on a
    /// shared source line; it only removes whitespace before a source newline
    /// or after one.
    fn literal_text(&self, literal: yag_template_syntax::ast::Text, sequence_end: Option<usize>) -> Doc {
        let range = literal.text_range();
        let start = byte_offset(range.start());
        let end = byte_offset(range.end());
        let Some(sequence_end) = sequence_end else {
            return text(literal.get());
        };

        let mut documents = Vec::new();
        let mut at_line_start = start == 0 || self.source.as_bytes()[start - 1] == b'\n';
        let mut consumed = 0;
        let terminal_newline_end = (end == sequence_end)
            .then(|| final_line_break_end(literal.get()))
            .flatten();
        for segment in literal.get().split_inclusive('\n') {
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
                documents.push(text(content));
            }
            if has_newline {
                let terminal_newline = terminal_newline_end == Some(consumed);
                if !terminal_newline {
                    documents.push(line());
                }
                at_line_start = true;
            } else {
                at_line_start = false;
            }
        }
        concat(documents)
    }

    fn policy_at_offset(&self, offset: usize) -> LayoutPolicy {
        self.plan.policy_at_offset(offset)
    }
}

/// Normalize only same-line whitespace adjacent to the delimiters of a
/// source-preserved multi-line action. Newline-led and newline-terminated
/// bodies retain their existing vertical layout.
fn format_multiline_delimited_body(
    body: &str,
    left_has_trim_marker: bool,
    right_has_trim_marker: bool,
    delimiter_padding: DelimiterPadding,
) -> String {
    let body = body.trim_start_matches(is_horizontal_whitespace);
    let starts_on_same_line = !starts_with_line_break(body);
    let body = body.trim_end_matches(is_horizontal_whitespace);
    let ends_on_same_line = !body.ends_with('\n');
    let padding = match delimiter_padding {
        DelimiterPadding::None => "",
        DelimiterPadding::Spaces => " ",
    };

    let mut formatted = String::with_capacity(body.len() + 2);
    if starts_on_same_line && !left_has_trim_marker {
        formatted.push_str(padding);
    }
    formatted.push_str(body);
    if ends_on_same_line && !right_has_trim_marker {
        formatted.push_str(padding);
    }
    formatted
}

fn is_horizontal_whitespace(character: char) -> bool {
    matches!(character, ' ' | '\t')
}

fn starts_with_line_break(text: &str) -> bool {
    text.starts_with('\n') || text.starts_with("\r\n")
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
