//! A small Wadler/Leijen-style document algebra.

use crate::Indent;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) enum Doc {
    /// A formatter-generated run of output, such as a keyword, delimiter, or
    /// identifier. The renderer emits it unchanged, but unlike `Verbatim` it
    /// may be positioned within `Group`, `SoftLine`, and `Nest` layout decisions.
    Text(String),
    /// Source text that must be emitted verbatim.
    Verbatim(String),
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
    pub(super) fn text(text: impl Into<String>) -> Self {
        Self::Text(text.into())
    }

    pub(super) fn verbatim(text: impl Into<String>) -> Self {
        Self::Verbatim(text.into())
    }

    pub(super) fn concat(parts: impl IntoIterator<Item = Doc>) -> Self {
        Self::Concat(parts.into_iter().collect())
    }

    pub(super) fn group(doc: Doc) -> Self {
        Self::Group(Box::new(doc))
    }

    pub(super) fn nest(indent: Indent, doc: Doc) -> Self {
        Self::Nest(indent, Box::new(doc))
    }
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
            Doc::Text(text) | Doc::Verbatim(text) => append_text(&mut out, &mut column, &text),
            Doc::Concat(parts) => push_parts(&mut commands, &indent, mode, parts),
            Doc::Line => append_line(&mut out, &mut column, &indent),
            Doc::SoftLine if mode == Mode::Flat => {
                out.push(' ');
                column += 1;
            }
            Doc::SoftLine => append_line(&mut out, &mut column, &indent),
            Doc::Nest(extra, doc) => commands.push(Command::new(indented(&indent, extra), mode, *doc)),
            Doc::Group(doc) => {
                let mut probe = commands.clone();
                probe.push(Command::new(&indent, Mode::Flat, (*doc).clone()));
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
            Doc::Text(text) | Doc::Verbatim(text) => {
                if let Some((_, tail)) = text.rsplit_once('\n') {
                    remaining = width as isize - tail.chars().count() as isize;
                } else {
                    remaining -= text.chars().count() as isize;
                }
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
    use super::{Doc, render};
    use crate::Indent;

    #[test]
    fn doc_width_boundaries_choose_flat_or_broken_groups() {
        let doc = Doc::group(Doc::concat([Doc::text("hello"), Doc::SoftLine, Doc::text("world")]));
        assert_eq!(render(doc.clone(), 11), "hello world");
        assert_eq!(render(doc, 10), "hello\nworld");
    }

    #[test]
    fn doc_nesting_applies_after_a_soft_line_break() {
        let doc = Doc::group(Doc::concat([
            Doc::text("hello"),
            Doc::nest(Indent::Spaces(2), Doc::concat([Doc::SoftLine, Doc::text("world")])),
        ]));
        assert_eq!(render(doc, 10), "hello\n  world");
    }

    #[test]
    fn doc_hard_lines_always_break() {
        let doc = Doc::group(Doc::concat([Doc::text("left"), Doc::Line, Doc::text("right")]));
        assert_eq!(render(doc, 100), "left\nright");
    }

    #[test]
    fn doc_verbatim_content_is_rendered_exactly() {
        let literal = "line one\n  original indentation\nline three";
        assert_eq!(render(Doc::verbatim(literal), 1), literal);
    }

    #[test]
    fn doc_nesting_uses_tabs_when_configured() {
        let doc = Doc::concat([
            Doc::text("header"),
            Doc::nest(Indent::Tabs, Doc::concat([Doc::Line, Doc::text("body")])),
        ]);
        assert_eq!(render(doc, 100), "header\n\tbody");
    }
}
