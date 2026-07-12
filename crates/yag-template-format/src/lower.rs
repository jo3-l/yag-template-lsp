use std::ops::Range;

use yag_template_syntax::SyntaxNode;
use yag_template_syntax::ast::{ActionList, ActionOrText, AstNode, AstToken, LeftDelim, RightDelim, Root, Text};

use crate::iterutil::iter_with_neighbors;
use crate::line_protection::{LineProtection, ReflowPolicy};
use crate::pretty::{Doc, concat, empty, group, group_with_tail, if_break, line, nest, soft_line, text};
use crate::{FormatOptions, LayoutKind};

/// Lower a successfully parsed root into a renderable document.
pub(super) fn lower(root: &SyntaxNode, source: &str, options: &FormatOptions, protection: &LineProtection) -> Doc {
    let Some(root) = Root::cast(root.clone()) else {
        return text(source);
    };
    let mut formatter = Formatter::new(source, options, protection);
    let elements = root.actions_with_text().collect::<Vec<_>>();
    formatter
        .sequence(&elements, SequenceContext::Root, AllowCompact::No)
        .doc
}

/// Formatting context shared by the typed AST rules.
pub(crate) struct Formatter<'a> {
    source: &'a str,
    options: &'a FormatOptions,
    protection: &'a LineProtection,
}

/// Whether a source sequence may render in the formatter's compact layout.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum AllowCompact {
    Yes,
    No,
}

impl AllowCompact {
    pub(crate) fn is_allowed(self) -> bool {
        matches!(self, Self::Yes)
    }
}

/// How a sequence lowers its literal text tokens.
#[derive(Clone, Copy)]
enum SequenceContext {
    Root,
    Body,
}

/// A lowered sibling sequence with a final line boundary owned by its caller.
struct LoweredSequence {
    doc: Doc,
    trailing_line: bool,
}

/// Literal body text after line-margin normalization but before document
/// construction.
struct BodyText<'a> {
    fragments: Vec<BodyTextFragment<'a>>,
    has_terminal_newline: bool,
}

enum BodyTextFragment<'a> {
    Text(&'a str),
    Line,
}

impl<'a> Formatter<'a> {
    /// Build the context used for one lowering pass.
    fn new(source: &'a str, options: &'a FormatOptions, protection: &'a LineProtection) -> Self {
        Self {
            source,
            options,
            protection,
        }
    }

    pub(crate) fn function_layout(&self, name: &str) -> Option<LayoutKind> {
        self.options.function_layouts.by_name.get(name).copied()
    }

    /// Return whether sibling separators on the source line at `offset` may
    /// participate in reflow.
    fn is_flexible_line_at(&self, offset: usize) -> bool {
        self.protection.policy_at_offset(offset) == ReflowPolicy::Flexible
    }
}

impl<'a> Formatter<'a> {
    pub(crate) fn continuation(&self, doc: Doc) -> Doc {
        nest(self.options.continuation_indent, doc)
    }

    pub(crate) fn delimited(&self, delims: (LeftDelim, RightDelim), body: Doc) -> Option<Doc> {
        self.delimited_with_breaking_close(delims, body, false)
    }

    /// Format an action while optionally moving its closing delimiter after a
    /// broken body onto a line of its own.
    pub(crate) fn delimited_with_breaking_close(
        &self,
        (left_delim, right_delim): (LeftDelim, RightDelim),
        body: Doc,
        break_before_close: bool,
    ) -> Option<Doc> {
        let padding = self.options.delimiter_padding.as_str();
        let left_padding = if left_delim.has_trim_marker() { "" } else { padding };
        let right_padding = if right_delim.has_trim_marker() { "" } else { padding };

        let doc = concat([
            text(left_delim.syntax().text()),
            text(left_padding),
            if break_before_close {
                group_with_tail(body, if_break(line(), text(right_padding)))
            } else {
                concat([body, text(right_padding)])
            },
            text(right_delim.syntax().text()),
        ]);

        let left_offset = byte_offset(left_delim.text_range().start());
        if self.protection.policy_at_offset(left_offset) == ReflowPolicy::Protected {
            doc.flatten()
        } else if break_before_close {
            Some(doc)
        } else {
            Some(group(doc))
        }
    }
}

impl<'a> Formatter<'a> {
    /// Lower one typed compound body and apply its block indentation.
    pub(crate) fn body(&mut self, body: ActionList, allow_compact: AllowCompact) -> Doc {
        let elements = body.actions_with_text().collect::<Vec<_>>();

        let (left_flexible, inner, right_flexible) = self.decompose_body(allow_compact, &elements);
        let sequence = self.sequence(inner, SequenceContext::Body, allow_compact);
        concat([
            nest(
                self.options.indent,
                concat([
                    // These stay spaces while the enclosing block fits flat.
                    if left_flexible { soft_line() } else { empty() },
                    sequence.doc,
                ]),
            ),
            if right_flexible { soft_line() } else { empty() },
            if sequence.trailing_line { line() } else { empty() },
        ])
    }

    /// Split a body that allows compact layout into formatter-owned edge whitespace and the
    /// elements that retain their source-owned relationships.
    ///
    /// Each flexible edge is one inline whitespace text token on a flexible
    /// source line. The caller lowers an edge as a [`soft_line`] and lowers
    /// only `inner` through the normal sequence rules, ensuring an edge token
    /// is not emitted twice as literal text.
    fn decompose_body<'b>(
        &self,
        allow_compact: AllowCompact,
        elements: &'b [ActionOrText],
    ) -> (bool, &'b [ActionOrText], bool) {
        if !allow_compact.is_allowed() {
            return (false, elements, false);
        }

        let (right_flexible, inner) = match elements.split_last() {
            Some((ActionOrText::Text(text), prefix)) if self.is_flexible_inline_whitespace(text) => (true, prefix),
            _ => (false, elements),
        };
        let (left_flexible, inner) = match inner.split_first() {
            Some((ActionOrText::Text(text), tail)) if self.is_flexible_inline_whitespace(text) => (true, tail),
            _ => (false, inner),
        };

        (left_flexible, inner, right_flexible)
    }

    /// Whether `text` can serve as an edge in a body that allows compact
    /// layout.
    ///
    /// This uses the text token's source offset so it follows the line protector's final
    /// protected-versus-flexible decision.
    fn is_flexible_inline_whitespace(&self, text: &Text) -> bool {
        let s = text.get();
        let is_inline_whitespace = s.chars().all(|c| c != '\n' && c.is_whitespace());
        is_inline_whitespace && self.is_flexible_line_at(byte_offset(text.text_range().start()))
    }

    /// Lower one direct root or body sequence.
    ///
    /// The sequence owns relationships between siblings that no individual AST
    /// action can see. Flexible action separators are [`soft_line`]s in a
    /// body that allows compact layout and structural [`line`]s otherwise; all
    /// remaining text is passed to [`literal_text`]. This keeps action
    /// separation policy in one place while typed action rules remain
    /// responsible only for their own syntax.
    fn sequence(
        &mut self,
        elements: &[ActionOrText],
        context: SequenceContext,
        allow_compact: AllowCompact,
    ) -> LoweredSequence {
        let action_separator = if allow_compact.is_allowed() {
            soft_line()
        } else {
            line()
        };

        let mut parts: Vec<Doc> = Vec::new();
        let mut trailing_line = false;
        for (previous, element, next) in iter_with_neighbors(elements) {
            match element {
                ActionOrText::Action(action) => {
                    let left_edge_flexible = self.is_flexible_line_at(source_range(action).start);
                    if matches!(previous, Some(ActionOrText::Action(_))) && left_edge_flexible {
                        parts.push(action_separator.clone());
                    }
                    parts.push(self.action(action.clone()));
                }
                ActionOrText::Text(literal) => {
                    let separates_actions = matches!(previous, Some(ActionOrText::Action(_)))
                        && matches!(next, Some(ActionOrText::Action(_)));
                    if self.is_flexible_inline_whitespace(literal) && separates_actions {
                        parts.push(action_separator.clone());
                    } else {
                        let (doc, final_line) = match context {
                            SequenceContext::Root => (text(literal.get()), false),
                            SequenceContext::Body => self.lower_body_text(literal, next.is_none()),
                        };
                        trailing_line |= final_line;
                        parts.push(doc);
                    }
                }
            }
        }
        LoweredSequence {
            doc: concat(parts),
            trailing_line,
        }
    }
}

impl<'a> Formatter<'a> {
    /// Lower normalized body text and detach its final line boundary when the
    /// enclosing body owns it.
    fn lower_body_text(&self, literal: &Text, is_final: bool) -> (Doc, bool) {
        let start = byte_offset(literal.text_range().start());
        let starts_new_line = start == 0 || self.source.as_bytes()[start - 1] == b'\n';
        let mut body_text = split_body_text(literal.get(), starts_new_line);
        let trailing_line = is_final && body_text.has_terminal_newline;
        if trailing_line {
            let Some(BodyTextFragment::Line) = body_text.fragments.pop() else {
                unreachable!("a terminal body line break must have a line fragment");
            };
        }

        let doc = concat(body_text.fragments.into_iter().map(|fragment| match fragment {
            BodyTextFragment::Text(content) => text(content),
            BodyTextFragment::Line => line(),
        }));
        (doc, trailing_line)
    }
}

/// Split literal body text into normalized content and structural line
/// boundaries without constructing a document.
fn split_body_text(text: &str, starts_new_line: bool) -> BodyText<'_> {
    let mut fragments = Vec::new();
    let mut at_line_start = starts_new_line;
    for segment in text.split_inclusive('\n') {
        let has_newline = segment.ends_with('\n');
        let mut content = segment.strip_suffix('\n').unwrap_or(segment);
        if at_line_start {
            content = content.trim_start_matches(char::is_whitespace);
        }
        if has_newline {
            content = content.trim_end_matches(char::is_whitespace);
        }
        if !content.is_empty() {
            fragments.push(BodyTextFragment::Text(content));
        }
        if has_newline {
            fragments.push(BodyTextFragment::Line);
            at_line_start = true;
        } else {
            at_line_start = false;
        }
    }

    let has_terminal_newline = text
        .trim_end_matches(|c: char| c != '\n' && c.is_whitespace())
        .ends_with('\n');
    BodyText {
        fragments,
        has_terminal_newline,
    }
}

/// Convert an AST node's text range to byte offsets for slicing `source`.
fn source_range(node: &impl AstNode) -> Range<usize> {
    let range = node.text_range();
    byte_offset(range.start())..byte_offset(range.end())
}

/// Convert the syntax library's byte-based text position to `usize`.
fn byte_offset(position: impl Into<u32>) -> usize {
    position.into() as usize
}
