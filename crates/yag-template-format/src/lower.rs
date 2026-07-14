use std::ops::Range;

use yag_template_syntax::SyntaxNode;
use yag_template_syntax::ast::{AstNode, Root};

use crate::line_protection::{LineProtection, ReflowPolicy};
use crate::pretty::{Doc, indent, text};
use crate::{FormatOptions, LayoutKind};

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
    pub(crate) source: &'a str,
    pub(crate) options: &'a FormatOptions,
    pub(crate) protection: &'a LineProtection,
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

    /// Return the reflow policy for the source line containing `offset`.
    pub(crate) fn reflow_policy_at(&self, offset: usize) -> ReflowPolicy {
        self.protection.policy_at_offset(offset)
    }

    pub(crate) fn indent_if_broken(&self, doc: Doc) -> Doc {
        indent(self.options.continuation_indent, doc)
    }
}

/// Convert the syntax library's byte-based text position to `usize`.
pub(crate) fn byte_offset(position: impl Into<u32>) -> usize {
    position.into() as usize
}

/// Convert an AST node's text range to byte offsets for slicing source text.
pub(crate) fn source_range(node: &impl AstNode) -> Range<usize> {
    let range = node.text_range();
    byte_offset(range.start())..byte_offset(range.end())
}
