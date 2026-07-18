//! Document fragments rendered between template action delimiters.

use yag_template_syntax::ast::{LeftDelim, RightDelim};

use crate::DelimiterPadding;
use crate::lower::Formatter;
use crate::pretty::{Doc, GroupId, concat, empty, group_with_id, if_break, line, text};

/// Lowered content with metadata about its trailing closing boundary.
pub(super) struct DelimitedInner {
    pub(super) doc: Doc,
    /// The named group which, when broken, generates the content's final
    /// closing-delimiter row.
    pub(super) trailing_closing_group: Option<GroupId>,
}

impl Formatter<'_> {
    pub(super) fn delimited(
        &mut self,
        (left_delim, right_delim): (LeftDelim, RightDelim),
        body: DelimitedInner,
    ) -> Doc {
        let DelimitedInner {
            doc: body,
            trailing_closing_group,
        } = body;
        let pad_delimiters = self.options.delimiter_padding == DelimiterPadding::Spaces;

        // Opening padding is horizontal: trim delimiters require one space,
        // while ordinary delimiters follow the configured padding.
        let left = if left_delim.has_trim_marker() {
            text("{{- ")
        } else if pad_delimiters {
            text("{{ ")
        } else {
            text("{{")
        };

        // Closing padding is emitted only when the delimiter stays on the same
        // row. A generated newline replaces it, so no line ends in padding.
        let closing_padding = if right_delim.has_trim_marker() || pad_delimiters {
            text(" ")
        } else {
            empty()
        };
        let right = if right_delim.has_trim_marker() {
            text("-}}")
        } else {
            text("}}")
        };

        let action_id = self.new_group_id();
        let action_boundary = if_break(action_id, line(), closing_padding.clone());
        let closing_boundary = trailing_closing_group.map_or(action_boundary.clone(), |closing_id| {
            // Reuse a trailing parenthesis row when one exists; otherwise the
            // action group decides whether the right delimiter needs a new row.
            if_break(closing_id, closing_padding, action_boundary)
        });
        group_with_id(action_id, concat([left, body, closing_boundary, right]))
    }
}

impl DelimitedInner {
    pub(super) fn new(doc: Doc) -> Self {
        Self {
            doc,
            trailing_closing_group: None,
        }
    }

    pub(super) fn with_prefix(self, prefix: Doc) -> Self {
        Self {
            doc: concat([prefix, self.doc]),
            // Prefixing does not change where the content ends.
            trailing_closing_group: self.trailing_closing_group,
        }
    }

    pub(super) fn with_suffix(self, suffix: Doc) -> Self {
        // Appending syntax means the content no longer ends at an earlier closing row.
        Self::new(concat([self.doc, suffix]))
    }

    pub(super) fn into_doc(self) -> Doc {
        self.doc
    }
}
