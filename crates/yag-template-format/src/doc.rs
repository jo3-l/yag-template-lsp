//! A small Wadler/Leijen-style document algebra.

use crate::Indent;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) enum Doc {
    /// A run of output whose surrounding whitespace and line breaks belong to
    /// the enclosing document.
    Text(String),
    /// Render each child in order.
    Concat(Vec<Doc>),
    /// An unconditional newline. Unlike `SoftLine`, it never flattens to a
    /// space when enclosed by a `Group`.
    Line,
    /// A conditional line break: one space in a flat `Group`, or a newline in
    /// a broken group.
    SoftLine,
    /// Attempt to render the enclosed document on the current line. The
    /// renderer selects flat or broken mode using its bounded `fits` probe.
    ///
    /// For example, `Group(Text("hello") + SoftLine + Text("world"))`
    /// becomes:
    ///
    /// ```text
    /// hello world
    /// ```
    ///
    /// when it fits, or two lines:
    ///
    /// ```text
    /// hello
    /// world
    /// ```
    ///
    /// when it does not.
    Group(Box<Doc>),
    /// Add configured indentation to the indentation applied after line breaks in the enclosed
    /// document. It has no effect while the document remains flat. For
    /// example, nesting `SoftLine + Text("world")` by two under `hello`
    /// produces:
    ///
    /// ```text
    /// hello world
    /// ```
    ///
    /// when flat, and
    ///
    /// ```text
    /// hello
    ///   world
    /// ```
    ///
    /// when broken.
    Nest(Indent, Box<Doc>),
}

impl Doc {
    /// Convert conditional layout to a single flat line. Hard lines and
    /// embedded newlines cannot safely be flattened.
    pub(super) fn flatten(self) -> Option<Self> {
        match self {
            Self::Text(text) => (!text.contains('\n')).then_some(Self::Text(text)),
            Self::Concat(parts) => parts
                .into_iter()
                .map(Self::flatten)
                .collect::<Option<Vec<_>>>()
                .map(Self::Concat),
            Self::Line => None,
            Self::SoftLine => Some(text(" ")),
            Self::Group(doc) | Self::Nest(_, doc) => doc.flatten(),
        }
    }
}

/// A normal layout-owned atom.
pub(super) fn text(text: impl Into<String>) -> Doc {
    Doc::Text(text.into())
}

/// Concatenate document fragments in order.
pub(super) fn concat(parts: impl IntoIterator<Item = Doc>) -> Doc {
    Doc::Concat(parts.into_iter().collect())
}

/// Concatenate only successful document fragments, propagating failure to the
/// enclosing typed rule when any fragment cannot be constructed.
pub(super) fn try_concat(parts: impl IntoIterator<Item = Option<Doc>>) -> Option<Doc> {
    parts.into_iter().collect::<Option<Vec<_>>>().map(concat)
}

/// Join document fragments with `separator`, returning an empty document for
/// an empty iterator.
pub(super) fn join(separator: Doc, parts: impl IntoIterator<Item = Doc>) -> Doc {
    let mut parts = parts.into_iter();
    let Some(first) = parts.next() else {
        return empty();
    };
    concat(std::iter::once(first).chain(parts.flat_map(|part| [separator.clone(), part])))
}

/// An empty document fragment.
pub(super) fn empty() -> Doc {
    concat([])
}

/// An unconditional line break.
pub(super) fn line() -> Doc {
    Doc::Line
}

/// A conditional line break that becomes a space when its group fits.
pub(super) fn soft_line() -> Doc {
    Doc::SoftLine
}

/// Attempt to render `doc` on one line before breaking it.
pub(super) fn group(doc: Doc) -> Doc {
    Doc::Group(Box::new(doc))
}

/// Apply `indent` after line breaks inside `doc`.
pub(super) fn nest(indent: Indent, doc: Doc) -> Doc {
    Doc::Nest(indent, Box::new(doc))
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum Mode {
    Flat,
    Break,
}

#[derive(Debug, Clone)]
struct Command {
    indent: String,
    mode: Mode,
    doc: Doc,
}

impl Command {
    fn new(indent: impl Into<String>, mode: Mode, doc: Doc) -> Self {
        Self {
            indent: indent.into(),
            mode,
            doc,
        }
    }
}

/// Render a document at `width`, probing a group in flat mode only while its
/// contents fit on the current line.
pub(super) fn render(doc: Doc, width: usize) -> String {
    let mut out = String::new();
    let mut column = 0;
    let mut commands = vec![Command::new("", Mode::Break, doc)];

    while let Some(Command { indent, mode, doc }) = commands.pop() {
        match doc {
            Doc::Text(text) => append_text(&mut out, &mut column, &text),
            Doc::Concat(parts) => push_parts(&mut commands, &indent, mode, parts),
            Doc::Line => append_line(&mut out, &mut column, &indent),
            Doc::SoftLine if mode == Mode::Flat => {
                out.push(' ');
                column += 1;
            }
            Doc::SoftLine => append_line(&mut out, &mut column, &indent),
            Doc::Nest(extra, doc) => commands.push(Command::new(indented(&indent, extra), mode, *doc)),
            Doc::Group(doc) => {
                // A group's decision belongs to that group. Looking through
                // later sibling documents makes short nested calls wrap only
                // because an unrelated action shares their source line.
                let probe = vec![Command::new(&indent, Mode::Flat, (*doc).clone())];
                let mode = if fits(width.saturating_sub(column), probe) {
                    Mode::Flat
                } else {
                    Mode::Break
                };
                commands.push(Command::new(indent, mode, *doc));
            }
        }
    }
    out
}

fn push_parts(commands: &mut Vec<Command>, indent: &str, mode: Mode, parts: Vec<Doc>) {
    commands.extend(parts.into_iter().rev().map(|part| Command::new(indent, mode, part)));
}

fn append_text(out: &mut String, column: &mut usize, text: &str) {
    out.push_str(text);
    *column = text
        .rsplit_once('\n')
        .map_or(*column + text.chars().count(), |(_, tail)| tail.chars().count());
}

fn append_line(out: &mut String, column: &mut usize, indent: &str) {
    out.push('\n');
    out.push_str(indent);
    *column = indent.chars().count();
}

fn indented(existing: &str, extra: Indent) -> String {
    let mut indent = existing.to_owned();
    match extra {
        Indent::Tabs => indent.push('\t'),
        Indent::Spaces(count) => indent.push_str(&" ".repeat(usize::from(count))),
    }
    indent
}

/// Bounded flat-mode probe. A hard or broken line always fits because the
/// next output segment starts at a fresh line.
fn fits(width: usize, mut commands: Vec<Command>) -> bool {
    let mut remaining = width as isize;
    while remaining >= 0 {
        let Some(Command { indent, mode, doc }) = commands.pop() else {
            return true;
        };
        match doc {
            Doc::Text(text) => {
                if text.contains('\n') {
                    return true;
                }
                remaining -= text.chars().count() as isize;
            }
            Doc::Concat(parts) => push_parts(&mut commands, &indent, mode, parts),
            Doc::Line => return true,
            Doc::SoftLine if mode == Mode::Flat => remaining -= 1,
            Doc::SoftLine => return true,
            Doc::Nest(extra, doc) => commands.push(Command::new(indented(&indent, extra), mode, *doc)),
            Doc::Group(doc) => commands.push(Command::new(indent, Mode::Flat, *doc)),
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::{concat, group, join, line, nest, render, soft_line, text, try_concat};
    use crate::Indent;

    #[test]
    fn doc_width_boundaries_choose_flat_or_broken_groups() {
        let doc = group(concat([text("hello"), soft_line(), text("world")]));
        assert_eq!(render(doc.clone(), 11), "hello world");
        assert_eq!(render(doc, 10), "hello\nworld");
    }

    #[test]
    fn doc_nesting_applies_after_a_soft_line_break() {
        let doc = group(concat([
            text("hello"),
            nest(Indent::Spaces(2), concat([soft_line(), text("world")])),
        ]));
        assert_eq!(render(doc, 10), "hello\n  world");
    }

    #[test]
    fn doc_hard_lines_always_break() {
        let doc = group(concat([text("left"), line(), text("right")]));
        assert_eq!(render(doc, 100), "left\nright");
    }

    #[test]
    fn doc_text_with_embedded_newlines_is_rendered_exactly() {
        let literal = "line one\n  original indentation\nline three";
        assert_eq!(render(text(literal), 1), literal);
    }

    #[test]
    fn doc_nesting_uses_tabs_when_configured() {
        let doc = concat([text("header"), nest(Indent::Tabs, concat([line(), text("body")]))]);
        assert_eq!(render(doc, 100), "header\n\tbody");
    }

    #[test]
    fn flatten_converts_soft_lines_and_refuses_hard_lines() {
        assert_eq!(
            concat([text("left"), soft_line(), text("right")]).flatten(),
            Some(concat([text("left"), text(" "), text("right")]))
        );
        assert_eq!(line().flatten(), None);
    }

    #[test]
    fn terse_helpers_compose_text_and_optional_fragments() {
        assert_eq!(render(concat([text("range "), text("$items")]), 100), "range $items");
        assert_eq!(
            render(join(text(", "), [text("$first"), text("$second")]), 100),
            "$first, $second"
        );
        assert_eq!(
            render(try_concat([Some(text("left")), Some(text("right"))]).unwrap(), 100),
            "leftright"
        );
        assert!(try_concat([Some(text("left")), None]).is_none());
    }
}
