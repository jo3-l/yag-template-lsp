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
    /// Select `broken` when enclosed by a broken `Group`, otherwise select
    /// `flat`.
    IfBreak { broken: Box<Doc>, flat: Box<Doc> },
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
    /// Like [`Group`], but the layout decision is based only on `body` while
    /// `tail` shares its selected mode. This lets a closing delimiter follow
    /// the body's layout without making that delimiter affect wrapping.
    GroupWithTail { body: Box<Doc>, tail: Box<Doc> },
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
    /// Wrap this document in a group when its parent decides compact layout.
    pub(super) fn group_if(self, allow_compact: AllowCompact) -> Self {
        match allow_compact {
            AllowCompact::Yes => Self::Group(Box::new(self)),
            AllowCompact::No => self,
        }
    }

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
            Self::IfBreak { flat, .. } => flat.flatten(),
            Self::Group(doc) | Self::Nest(_, doc) => doc.flatten(),
            Self::GroupWithTail { body, tail } => Some(concat([body.flatten()?, tail.flatten()?])),
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

/// Choose a document based on the enclosing group's layout mode.
pub(super) fn if_break(broken: Doc, flat: Doc) -> Doc {
    Doc::IfBreak {
        broken: Box::new(broken),
        flat: Box::new(flat),
    }
}

/// Attempt to render `doc` on one line before breaking it.
pub(super) fn group(doc: Doc) -> Doc {
    Doc::Group(Box::new(doc))
}

/// Choose the layout from `body`, then render `tail` in that same layout.
pub(super) fn group_with_tail(body: Doc, tail: Doc) -> Doc {
    Doc::GroupWithTail {
        body: Box::new(body),
        tail: Box::new(tail),
    }
}

/// Apply `indent` after line breaks inside `doc`.
pub(super) fn nest(indent: Indent, doc: Doc) -> Doc {
    Doc::Nest(indent, Box::new(doc))
}

#[cfg(test)]
mod tests {
    use super::{concat, group, line, render, soft_line, text};

    #[test]
    fn flatten_converts_soft_lines_and_refuses_hard_lines() {
        assert_eq!(
            concat([text("left"), soft_line(), text("right")]).flatten(),
            Some(concat([text("left"), text(" "), text("right")]))
        );
        assert_eq!(line().flatten(), None);
    }

    #[test]
    fn helpers_compose_document_fragments() {
        let doc = group(concat([text("hello"), soft_line(), text("world")]));
        assert_eq!(render(doc, 11), "hello world");
    }
}
