//! Iterative renderer and bounded flat-layout probe.

use super::Doc;
use crate::Indent;

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
pub(crate) fn render(doc: Doc, width: usize) -> String {
    let mut out = String::new();
    let mut column = 0;
    let mut pending_indent = None;
    let mut commands = vec![Command::new("", Mode::Break, doc)];

    while let Some(Command { indent, mode, doc }) = commands.pop() {
        match doc {
            Doc::Text(text) => {
                append_pending_indent(&mut out, &mut column, &mut pending_indent);
                append_text(&mut out, &mut column, &text);
            }
            Doc::Concat(parts) => push_parts(&mut commands, &indent, mode, parts),
            Doc::Line => append_line(&mut out, &mut column, &mut pending_indent, indent),
            Doc::SoftLine if mode == Mode::Flat => {
                append_pending_indent(&mut out, &mut column, &mut pending_indent);
                out.push(' ');
                column += 1;
            }
            Doc::SoftLine => append_line(&mut out, &mut column, &mut pending_indent, indent),
            Doc::Indent(extra, doc) => commands.push(Command::new(indented(&indent, extra), mode, *doc)),
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

fn append_pending_indent(out: &mut String, column: &mut usize, pending_indent: &mut Option<String>) {
    if let Some(indent) = pending_indent.take() {
        out.push_str(&indent);
        *column = indent.chars().count();
    }
}

fn append_line(out: &mut String, column: &mut usize, pending_indent: &mut Option<String>, indent: String) {
    out.push('\n');
    *column = indent.chars().count();
    *pending_indent = Some(indent);
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
            Doc::Indent(extra, doc) => commands.push(Command::new(indented(&indent, extra), mode, *doc)),
            Doc::Group(doc) => commands.push(Command::new(indent, Mode::Flat, *doc)),
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::render;
    use crate::Indent;
    use crate::pretty::{concat, group, indent, join, line, soft_line, text, try_concat};

    #[test]
    fn width_boundaries_choose_flat_or_broken_groups() {
        let doc = group(concat([text("hello"), soft_line(), text("world")]));
        assert_eq!(render(doc.clone(), 11), "hello world");
        assert_eq!(render(doc, 10), "hello\nworld");
    }

    #[test]
    fn indentation_applies_after_a_soft_line_break() {
        let doc = group(concat([
            text("hello"),
            indent(Indent::Spaces(2), concat([soft_line(), text("world")])),
        ]));
        assert_eq!(render(doc, 10), "hello\n  world");
    }

    #[test]
    fn hard_lines_always_break() {
        let doc = group(concat([text("left"), line(), text("right")]));
        assert_eq!(render(doc, 100), "left\nright");
    }

    #[test]
    fn text_with_embedded_newlines_is_rendered_exactly() {
        let literal = "line one\n  original indentation\nline three";
        assert_eq!(render(text(literal), 1), literal);
    }

    #[test]
    fn indentation_uses_tabs_when_configured() {
        let doc = concat([text("header"), indent(Indent::Tabs, concat([line(), text("body")]))]);
        assert_eq!(render(doc, 100), "header\n\tbody");
    }

    #[test]
    fn indentation_does_not_indent_empty_lines() {
        let doc = concat([
            text("header"),
            indent(
                Indent::Tabs,
                concat([line(), text("body"), line(), line(), text("tail")]),
            ),
        ]);
        assert_eq!(render(doc, 100), "header\n\tbody\n\n\ttail");
    }

    #[test]
    fn helpers_compose_text_and_optional_fragments() {
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
