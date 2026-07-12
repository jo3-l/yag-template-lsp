use std::ops::Range;

use yag_template_syntax::SyntaxNode;
use yag_template_syntax::ast::{
    Action, ActionList, ActionOrText, AstNode, AstToken, LeftDelim, RightDelim, Root, Text,
};

use crate::classification::{LayoutPolicy, LinePlan};
use crate::pretty::{Doc, concat, empty, group, group_with_tail, if_break, line, nest, soft_line, text};
use crate::{DelimiterPadding, FormatOptions, LayoutKind};

/// Lower a successfully parsed root into a renderable document.
pub(super) fn lower(root: &SyntaxNode, source: &str, options: &FormatOptions, plan: &LinePlan) -> Doc {
    let Some(root) = Root::cast(root.clone()) else {
        return text(source);
    };
    let mut formatter = Formatter::new(source, options, plan);
    formatter.sequence(root.actions_with_text(), None)
}

/// Formatting context shared by the typed AST rules.
pub(crate) struct Formatter<'a> {
    source: &'a str,
    options: &'a FormatOptions,
    plan: &'a LinePlan,
}

/// Layout-owned whitespace at the two edges of a compact compound body.
/// The surrounding group renders each edge as either a space or a line break.
#[derive(Debug, Default, Clone, Copy)]
struct BodyEdges {
    leading_flexible: bool,
    trailing_flexible: bool,
}

impl BodyEdges {
    /// Iterate over body elements after removing layout-owned edge text.
    ///
    /// The removed text is represented by a [`soft_line`] in
    /// [`Formatter::body`], so it is either restored as a space or rendered as
    /// a structural line by the enclosing compact block group.
    fn interior<'a>(self, elements: &'a [ActionOrText]) -> impl Iterator<Item = ActionOrText> + 'a {
        let start = usize::from(self.leading_flexible);
        let end = elements.len() - usize::from(self.trailing_flexible);
        elements[start..end].iter().cloned()
    }
}

impl<'a> Formatter<'a> {
    /// Build the context used for one lowering pass.
    fn new(source: &'a str, options: &'a FormatOptions, plan: &'a LinePlan) -> Self {
        Self { source, options, plan }
    }

    pub(crate) fn function_layout(&self, name: &str) -> Option<LayoutKind> {
        self.options.function_layouts.by_name.get(name).copied()
    }

    /// Nest a continuation beneath the current expression position.
    ///
    /// Expression rules use this around a leading [`soft_line`] so the same
    /// document is a normal space while flat and gains configured continuation
    /// indentation when its group breaks.
    pub(crate) fn continuation(&self, doc: Doc) -> Doc {
        nest(self.options.continuation_indent, doc)
    }

    /// Format a normal parsed action with its explicit delimiter tokens.
    pub(crate) fn delimited(&self, (left, right): (LeftDelim, RightDelim), body: Doc) -> Option<Doc> {
        self.delimited_with_breaking_close((left, right), body, false)
    }

    /// Format an action while optionally moving its closing delimiter after a
    /// broken body onto a line of its own.
    ///
    /// Delimiter token spelling comes from the parsed source so trim markers
    /// remain exact, but the action body always comes from the typed document.
    /// Existing internal action whitespace, including newlines, never changes
    /// which layout path is used.
    pub(crate) fn delimited_with_breaking_close(
        &self,
        (left, right): (LeftDelim, RightDelim),
        body: Doc,
        break_before_close: bool,
    ) -> Option<Doc> {
        let left_offset = byte_offset(left.text_range().start());
        let padding = match self.options.delimiter_padding {
            DelimiterPadding::None => "",
            DelimiterPadding::Spaces => " ",
        };
        let left_padding = if left.has_trim_marker() { "" } else { padding };
        let right_padding = if right.has_trim_marker() { "" } else { padding };
        let body = if break_before_close {
            group_with_tail(body, if_break(line(), text(right_padding)))
        } else {
            body
        };
        let doc = concat([
            text(left.syntax().text()),
            // A trim marker token owns its grammar-required space. Configured
            // delimiter padding therefore applies only to its ordinary peer.
            text(left_padding),
            body,
            if break_before_close {
                empty()
            } else {
                text(right_padding)
            },
            text(right.syntax().text()),
        ]);
        if self.policy_at_offset(left_offset) == LayoutPolicy::Protected {
            doc.flatten()
        } else if break_before_close {
            Some(doc)
        } else {
            Some(group(doc))
        }
    }

    /// Lower one typed compound body and apply its block indentation.
    pub(crate) fn body(&mut self, body: ActionList, compact: bool) -> Doc {
        let range = source_range(&body);
        let sequence_end = range.end;
        let has_terminal_newline = ends_with_line_break_after_margin(&self.source[range]);
        let elements = body.actions_with_text().collect::<Vec<_>>();
        let edges = self.compact_body_edges(compact, &elements);
        let content = nest(
            self.options.indent,
            concat([
                // These stay spaces while the enclosing compact block fits.
                if edges.leading_flexible { soft_line() } else { empty() },
                self.sequence(edges.interior(&elements), Some(sequence_end)),
            ]),
        );
        concat([
            content,
            if edges.trailing_flexible { soft_line() } else { empty() },
            if has_terminal_newline { line() } else { empty() },
        ])
    }

    /// Identify the whitespace tokens that a compact block may reflow.
    ///
    /// Only an outer body edge made of one inline whitespace token beside a
    /// flexible action is eligible. A non-compact source block, protected
    /// display action, literal text, or existing newline leaves the edge in the
    /// normal sequence unchanged.
    fn compact_body_edges(&self, compact: bool, elements: &[ActionOrText]) -> BodyEdges {
        if !compact {
            return BodyEdges::default();
        }
        let leading = matches!(
            elements,
            [ActionOrText::Text(text), ActionOrText::Action(action), ..]
                if self.is_flexible_inline_separator(text.get(), action)
        );
        let trailing = matches!(
            elements,
            [.., ActionOrText::Action(action), ActionOrText::Text(text)]
                if self.is_flexible_inline_separator(text.get(), action)
        );
        BodyEdges {
            leading_flexible: leading,
            trailing_flexible: trailing,
        }
    }

    /// Whether `text` can serve as a compact body edge before or after `action`.
    ///
    /// This uses the action's source offset rather than its syntax shape so it
    /// follows the line classifier's final protected-versus-flexible decision.
    fn is_flexible_inline_separator(&self, text: &str, action: &Action) -> bool {
        is_inline_whitespace(text) && self.has_flexible_layout_at(source_range(action).start)
    }

    /// Lower one direct root or body sequence.
    ///
    /// The sequence owns relationships between siblings that no individual AST
    /// action can see. Consecutive flexible actions gain a structural
    /// [`line`]; whitespace-only text between flexible actions is replaced by
    /// the same line; all remaining text is passed to [`literal_text`]. This
    /// keeps action separation policy in one place while typed action rules
    /// remain responsible only for their own syntax.
    fn sequence(&mut self, elements: impl Iterator<Item = ActionOrText>, sequence_end: Option<usize>) -> Doc {
        let elements = elements.collect::<Vec<_>>();
        let mut documents = Vec::new();
        for (index, element) in elements.iter().cloned().enumerate() {
            match element {
                ActionOrText::Action(action) => {
                    if self.should_break_before_action(&elements, index, &action) {
                        documents.push(line());
                    }
                    documents.push(self.action(action));
                }
                ActionOrText::Text(text) => {
                    if self.should_break_at_text_separator(&elements, index, &text) {
                        documents.push(line());
                    } else {
                        documents.push(self.literal_text(text, sequence_end));
                    }
                }
            }
        }
        concat(documents)
    }

    /// Decide whether an action directly following another action starts a new
    /// formatter-owned line.
    fn should_break_before_action(&self, elements: &[ActionOrText], index: usize, action: &Action) -> bool {
        index > 0
            && matches!(elements[index - 1], ActionOrText::Action(_))
            && self.has_flexible_layout_at(source_range(action).start)
    }

    /// Decide whether a text token is an inline, formatter-owned separator
    /// between two actions.
    ///
    /// The policy is queried at the token's source offset. This matters when a
    /// protected display action shares the physical line with a flexible action:
    /// protected ownership wins for the whole line.
    fn should_break_at_text_separator(&self, elements: &[ActionOrText], index: usize, separator: &Text) -> bool {
        if !is_inline_whitespace(separator.get()) || index == 0 {
            return false;
        }
        matches!(elements[index - 1], ActionOrText::Action(_))
            && matches!(elements.get(index + 1), Some(ActionOrText::Action(_)))
            && self.has_flexible_layout_at(byte_offset(separator.text_range().start()))
    }

    /// Lower literal text while normalizing only physical line margins in a
    /// compound body.
    ///
    /// Root-level text is emitted verbatim. Within a body, each source line is
    /// split at existing newlines: leading whitespace is removed only at a
    /// physical line start, trailing whitespace only before a newline, and the
    /// newline itself becomes a [`line`]. Text on a shared action/text line is
    /// retained exactly, so this never invents or removes same-line adjacency.
    fn literal_text(&self, literal: Text, sequence_end: Option<usize>) -> Doc {
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

    /// Return whether the line classifier grants formatter layout ownership at
    /// `offset`.
    fn has_flexible_layout_at(&self, offset: usize) -> bool {
        self.policy_at_offset(offset) == LayoutPolicy::Flexible
    }

    /// Read the resolved source-line ownership at `offset`.
    fn policy_at_offset(&self, offset: usize) -> LayoutPolicy {
        self.plan.policy_at_offset(offset)
    }
}

/// Return whether `text` is whitespace confined to one physical source line.
/// Such text can be a formatter-owned action separator; any newline keeps it
/// structural and source-owned.
fn is_inline_whitespace(text: &str) -> bool {
    !text.contains('\n') && text.chars().all(char::is_whitespace)
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

/// Return whether `text` ends with a newline followed only by line-margin
/// whitespace.
fn ends_with_line_break_after_margin(text: &str) -> bool {
    final_line_break_end(text).is_some()
}

/// The byte offset immediately after a final newline followed only by physical
/// line-margin whitespace.
fn final_line_break_end(text: &str) -> Option<usize> {
    let without_margin = text.trim_end_matches(|character: char| character != '\n' && character.is_whitespace());
    without_margin.ends_with('\n').then_some(without_margin.len())
}
