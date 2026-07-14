use yag_template_syntax::SyntaxNode;
use yag_template_syntax::ast::{AstNode, LeftDelim, RightDelim, Root};

use crate::line_protection::{LineProtection, ReflowPolicy};
use crate::pretty::{Doc, concat, empty, group, nest, soft_line, text};
use crate::{DelimiterPadding, FormatOptions, LayoutKind};

/// Lower a successfully parsed root into a renderable document.
pub(super) fn lower(root: &SyntaxNode, source: &str, options: &FormatOptions, protection: &LineProtection) -> Doc {
    let Some(root) = Root::cast(root.clone()) else {
        return text(source);
    };
    let mut formatter = Formatter::new(source, options, protection);
    let elements = root.actions_with_text().collect::<Vec<_>>();
    formatter.root(&elements)
}

/// Formatting context shared by the typed AST rules.
pub(crate) struct Formatter<'a> {
    source: &'a str,
    options: &'a FormatOptions,
    protection: &'a LineProtection,
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

    /// Return the immutable configuration for this lowering pass.
    pub(crate) fn options(&self) -> &FormatOptions {
        self.options
    }

    /// Return the source text associated with this lowering pass.
    pub(crate) fn source(&self) -> &'a str {
        self.source
    }

    /// Return the reflow policy for the source line containing `offset`.
    pub(crate) fn reflow_policy_at(&self, offset: usize) -> ReflowPolicy {
        self.protection.policy_at_offset(offset)
    }
}

impl<'a> Formatter<'a> {
    pub(crate) fn continuation(&self, doc: Doc) -> Doc {
        nest(self.options.continuation_indent, doc)
    }

    pub(crate) fn delimited(&self, (left_delim, right_delim): (LeftDelim, RightDelim), body: Doc) -> Doc {
        let pad_delimiters = self.options.delimiter_padding == DelimiterPadding::Spaces;
        let left = if left_delim.has_trim_marker() {
            concat([text("{{-"), soft_line()])
        } else {
            concat([text("{{"), if pad_delimiters { soft_line() } else { empty() }])
        };
        let right = if right_delim.has_trim_marker() {
            concat([soft_line(), text("-}}")])
        } else {
            concat([if pad_delimiters { soft_line() } else { empty() }, text("}}")])
        };

        group(concat([left, body, right]))
    }
}

/// Convert the syntax library's byte-based text position to `usize`.
pub(crate) fn byte_offset(position: impl Into<u32>) -> usize {
    position.into() as usize
}
