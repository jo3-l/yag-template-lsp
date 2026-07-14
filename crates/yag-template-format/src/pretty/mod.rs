//! Document algebra and construction helpers for pretty-printing.

use crate::Indent;

mod render;

pub(super) use render::render;

/// Whether a source sequence may render compactly on a single line.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum AllowCompact {
    Yes,
    No,
}

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
    /// example, indenting `SoftLine + Text("world")` by two under `hello`
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
    Indent(Indent, Box<Doc>),
}

impl Doc {
    /// Wrap this document in a group when its parent decides compact layout.
    pub(super) fn group_if(self, allow_compact: AllowCompact) -> Self {
        match allow_compact {
            AllowCompact::Yes => Self::Group(Box::new(self)),
            AllowCompact::No => self,
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
pub(super) fn indent(indent: Indent, doc: Doc) -> Doc {
    Doc::Indent(indent, Box::new(doc))
}

#[cfg(test)]
mod tests {
    use super::{concat, group, render, soft_line, text};

    #[test]
    fn helpers_compose_document_fragments() {
        let doc = group(concat([text("hello"), soft_line(), text("world")]));
        assert_eq!(render(doc, 11), "hello world");
    }
}
