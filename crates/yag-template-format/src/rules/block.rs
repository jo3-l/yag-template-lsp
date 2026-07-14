//! Declarative document rules for template blocks and compound bodies.

use yag_template_syntax::ast::{ActionList, ActionOrText, AstNode, AstToken, Text};

use crate::cursor::DoubleEndedPeekable;
use crate::iterutil::iter_with_neighbors;
use crate::line_protection::ReflowPolicy;
use crate::lower::{Formatter, byte_offset, source_range};
use crate::pretty::{AllowCompact, Doc, concat, empty, indent, line, soft_line, text};

/// How a sequence lowers its literal text tokens.
#[derive(Clone, Copy)]
enum SequenceContext {
    Root,
    Body,
}

/// A source boundary structurally eligible for formatter-owned reflow.
///
/// Literal text with content or source line breaks is deliberately excluded:
/// it remains source-owned. The constructors establish the source shape; the
/// formatter decides whether the boundary's line policy permits reflow.
struct ReflowBoundary {
    line_offset: usize,
}

impl ReflowBoundary {
    /// Construct a boundary only when `text` is whitespace confined to one
    /// physical source line.
    fn inline_whitespace(text: &Text) -> Option<Self> {
        is_inline_whitespace(text).then_some(Self {
            line_offset: byte_offset(text.text_range().start()),
        })
    }

    /// Construct the zero-width boundary immediately before an action.
    fn before(action: &impl AstNode) -> Self {
        Self {
            line_offset: source_range(action).start,
        }
    }

    /// Construct the zero-width boundary immediately after an action.
    fn after(action: &impl AstNode) -> Self {
        let range = source_range(action);
        Self {
            line_offset: range
                .end
                .checked_sub(1)
                .expect("template actions always have a non-empty source range"),
        }
    }
}

/// A lowered sibling sequence with a final line boundary owned by its caller.
struct LoweredSequence {
    doc: Doc,
    trailing_line: bool,
}

/// Formatter-owned reflow at a compound body's leading and trailing edges.
struct BodyEdgeReflow<'a> {
    leading: Doc,
    elements: &'a [ActionOrText],
    trailing: Doc,
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
    /// Lower the root action/text sequence.
    pub(crate) fn root(&mut self, elements: &[ActionOrText]) -> Doc {
        self.sequence(elements, SequenceContext::Root, AllowCompact::No).doc
    }

    /// Lower one typed compound body and apply its block indentation.
    pub(crate) fn body(&mut self, body: ActionList, allow_compact: AllowCompact) -> Doc {
        let elements = body.actions_with_text().collect::<Vec<_>>();

        let BodyEdgeReflow {
            leading,
            elements,
            trailing,
        } = self.reflow_body_edges(allow_compact, &elements);
        if elements.is_empty() {
            empty()
        } else {
            let lowered_inner = self.sequence(elements, SequenceContext::Body, allow_compact);
            concat([
                indent(self.options.indent, concat([leading, lowered_inner.doc])),
                trailing,
                if lowered_inner.trailing_line { line() } else { empty() },
            ])
        }
    }
}

impl<'a> Formatter<'a> {
    /// Reflow formatter-owned whitespace at a compound body's edges.
    ///
    /// Compact reflow turns flexible inline edge whitespace into soft lines.
    /// Expanded reflow turns only flexible action-to-clause adjacency into
    /// hard lines. Both preserve protected and source-owned edges.
    fn reflow_body_edges<'b>(&self, allow_compact: AllowCompact, elements: &'b [ActionOrText]) -> BodyEdgeReflow<'b> {
        match allow_compact {
            AllowCompact::Yes => self.reflow_compact_edges(elements),
            AllowCompact::No => self.reflow_expanded_edges(elements),
        }
    }

    /// Reflow edges for a body that may remain on one line.
    ///
    /// Inline whitespace beside a flexible first or last action belongs to the
    /// formatter and becomes a soft boundary. All other content remains in the
    /// body so literal text and source line breaks retain their relationships.
    fn reflow_compact_edges<'b>(&self, elements: &'b [ActionOrText]) -> BodyEdgeReflow<'b> {
        use ActionOrText::*;

        let mut cursor = DoubleEndedPeekable::new(elements);
        let leading = match cursor.peek_first() {
            Some(Text(text)) if self.may_reflow_inline_whitespace(text) => {
                // replace leading inline whitespace with soft newline
                cursor.drop_first();
                soft_line()
            }
            _ => empty(),
        };
        let trailing = match cursor.peek_last() {
            Some(Text(text)) if self.may_reflow_inline_whitespace(text) => {
                // replace trailing inline whitespace with soft newline
                cursor.drop_last();
                soft_line()
            }
            _ => empty(),
        };

        BodyEdgeReflow {
            leading,
            elements: cursor.remaining(),
            trailing,
        }
    }

    /// Reflow flexible edges for a body whose action boundaries are vertical.
    ///
    /// Direct action-to-clause adjacency becomes a hard boundary only on a
    /// flexible line. Literal text, source line breaks, and protected gaps stay
    /// source-owned to preserve their exact block layout.
    fn reflow_expanded_edges<'b>(&self, elements: &'b [ActionOrText]) -> BodyEdgeReflow<'b> {
        use ActionOrText::*;

        let mut cursor = DoubleEndedPeekable::new(elements);

        let leading = match (cursor.peek_first(), cursor.peek_second()) {
            (Some(Action(action)), _) if self.may_reflow_before(action) => line(),
            (Some(Text(text)), Some(Action(_))) if self.may_reflow_inline_whitespace(text) => {
                // replace leading inline whitespace with hard newline
                cursor.drop_first();
                line()
            }
            _ => empty(),
        };
        let trailing = match (cursor.peek_secondlast(), cursor.peek_last()) {
            (_, Some(Action(action))) if self.may_reflow_after(action) => line(),
            (Some(Action(_)), Some(Text(text))) if self.may_reflow_inline_whitespace(text) => {
                // replace trailing inline whitespace with hard newline
                cursor.drop_last();
                line()
            }
            _ => empty(),
        };

        BodyEdgeReflow {
            leading,
            elements: cursor.remaining(),
            trailing,
        }
    }

    /// Lower one direct root or body sequence.
    ///
    /// The sequence owns relationships between siblings that no individual AST
    /// action can see. Flexible action separators are [`soft_line`]s in a
    /// body that allows compact layout and structural [`line`]s otherwise; all
    /// remaining text is passed to [`lower_body_text`]. This keeps action
    /// separation policy in one place while typed action rules remain
    /// responsible only for their own syntax.
    fn sequence(
        &mut self,
        elements: &[ActionOrText],
        context: SequenceContext,
        allow_compact: AllowCompact,
    ) -> LoweredSequence {
        let action_separator = match allow_compact {
            AllowCompact::Yes => soft_line(),
            AllowCompact::No => line(),
        };

        let mut parts: Vec<Doc> = Vec::new();
        let mut trailing_line = false;
        for (previous, element, next) in iter_with_neighbors(elements) {
            match element {
                ActionOrText::Action(action) => {
                    if matches!(previous, Some(ActionOrText::Action(_))) && self.may_reflow_before(action) {
                        parts.push(action_separator.clone());
                    }
                    parts.push(self.action(action.clone()));
                }
                ActionOrText::Text(literal) => {
                    let separates_actions = matches!(previous, Some(ActionOrText::Action(_)))
                        && matches!(next, Some(ActionOrText::Action(_)));
                    if separates_actions && self.may_reflow_inline_whitespace(literal) {
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

    /// Return whether this exact source boundary belongs to formatter-owned
    /// reflow. Protected boundaries always remain source-owned.
    fn may_reflow_boundary(&self, boundary: ReflowBoundary) -> bool {
        self.reflow_policy_at(boundary.line_offset) == ReflowPolicy::Flexible
    }

    /// Return whether `text` is an inline-whitespace boundary on a flexible
    /// source line.
    fn may_reflow_inline_whitespace(&self, text: &Text) -> bool {
        ReflowBoundary::inline_whitespace(text).is_some_and(|boundary| self.may_reflow_boundary(boundary))
    }

    /// Return whether a direct boundary immediately before `action` may be
    /// reflowed. The boundary is on the action's opening-delimiter line.
    fn may_reflow_before(&self, action: &impl AstNode) -> bool {
        self.may_reflow_boundary(ReflowBoundary::before(action))
    }

    /// Return whether a direct boundary immediately after `action` may be
    /// reflowed. A multiline action can end on a different physical line than
    /// it starts, so this uses its final source byte rather than its start.
    fn may_reflow_after(&self, action: &impl AstNode) -> bool {
        self.may_reflow_boundary(ReflowBoundary::after(action))
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

/// Whether `text` is whitespace confined to one physical line.
fn is_inline_whitespace(text: &Text) -> bool {
    let text = text.get();
    !text.contains('\n') && text.chars().all(char::is_whitespace)
}
