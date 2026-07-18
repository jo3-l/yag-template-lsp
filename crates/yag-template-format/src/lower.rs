use std::ops::Range;

use yag_template_envdefs::EnvDefs;
use yag_template_syntax::SyntaxNode;
use yag_template_syntax::ast::{AstNode, Root};

use crate::FormatOptions;
use crate::line_protection::{LineProtection, ReflowPolicy};
use crate::pretty::{AllowCompact, Doc, GroupId, concat, indent, line, text};

/// Lower a successfully parsed root into a renderable document.
pub(super) fn lower(
    root: &SyntaxNode,
    source: &str,
    envdefs: &EnvDefs,
    options: &FormatOptions,
    protection: &LineProtection,
) -> Doc {
    let Some(root) = Root::cast(root.clone()) else {
        return text(source);
    };
    let mut f = Formatter::new(source, envdefs, options, protection);
    let body = root.actions_with_text().collect::<Vec<_>>();
    concat([f.sequence(&body, AllowCompact::No).doc, line()])
}

/// Formatting context shared by the typed AST rules.
pub(crate) struct Formatter<'a> {
    pub(crate) source: &'a str,
    pub(crate) envdefs: &'a EnvDefs,
    pub(crate) options: &'a FormatOptions,
    pub(crate) protection: &'a LineProtection,
    next_group_id: usize,
}

impl<'a> Formatter<'a> {
    /// Build the context used for one lowering pass.
    fn new(source: &'a str, envdefs: &'a EnvDefs, options: &'a FormatOptions, protection: &'a LineProtection) -> Self {
        Self {
            source,
            envdefs,
            options,
            protection,
            next_group_id: 0,
        }
    }

    /// Allocate a new named pretty-printing group.
    pub(crate) fn new_group_id(&mut self) -> GroupId {
        let id = GroupId(self.next_group_id);
        self.next_group_id += 1;
        id
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
