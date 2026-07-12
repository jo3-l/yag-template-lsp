//! A formatter for YAG templates.
//!
//! Invalid input is always returned unchanged. The current lowering stages
//! format delimiter padding, parsed block indentation, and ordinary expression
//! layout; specialized function layouts remain for later stages.

use std::collections::HashMap;

use yag_template_syntax::SyntaxNode;

mod classification;
mod line_index;
mod lower;
mod pretty;
mod rules;

/// Indentation used for template blocks or expression continuations.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Indent {
    Tabs,
    Spaces(u8),
}

/// Padding between ordinary action delimiters and their body.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DelimiterPadding {
    None,
    Spaces,
}

/// Layout dispatch table for calls with known syntactic callees.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FunctionLayouts {
    pub by_name: HashMap<String, LayoutKind>,
}

impl Default for FunctionLayouts {
    fn default() -> Self {
        Self {
            by_name: HashMap::from([
                ("dict".to_owned(), LayoutKind::KeyValuePairs),
                ("sdict".to_owned(), LayoutKind::KeyValuePairs),
            ]),
        }
    }
}

/// A configured function's document layout.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum LayoutKind {
    Call,
    KeyValuePairs,
}

/// Formatter configuration.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FormatOptions {
    pub indent: Indent,
    pub continuation_indent: Indent,
    pub max_width: usize,
    pub delimiter_padding: DelimiterPadding,
    pub function_layouts: FunctionLayouts,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            indent: Indent::Tabs,
            continuation_indent: Indent::Tabs,
            max_width: 100,
            delimiter_padding: DelimiterPadding::Spaces,
            function_layouts: FunctionLayouts::default(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FormatDiagnosticKind {
    ParseError,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FormatDiagnostic {
    pub kind: FormatDiagnosticKind,
    pub message: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FormatResult {
    pub text: String,
    pub diagnostics: Vec<FormatDiagnostic>,
}

/// Format `source` according to `options`.
///
/// Parse, classify, lower, and render `source` according to `options`.
pub fn format(source: &str, options: &FormatOptions) -> FormatResult {
    let parsed = yag_template_syntax::parser::parse(source);
    let diagnostics = parsed
        .errors
        .iter()
        .map(|error| FormatDiagnostic {
            kind: FormatDiagnosticKind::ParseError,
            message: error.to_string(),
        })
        .collect::<Vec<_>>();
    if parsed.errors.is_empty() {
        let root = SyntaxNode::new_root(parsed.root.clone());
        let line_plan = classification::classify(&root, source);
        let doc = lower::lower(&root, source, options, &line_plan);
        let text = pretty::render(doc, options.max_width);
        return FormatResult { text, diagnostics };
    }
    FormatResult {
        text: source.to_owned(),
        diagnostics,
    }
}

#[cfg(test)]
mod tests {
    use super::{DelimiterPadding, FormatOptions, format};

    #[test]
    fn formatting_overwidth_protected_text_has_no_diagnostics() {
        let options = FormatOptions {
            delimiter_padding: DelimiterPadding::Spaces,
            max_width: 8,
            ..FormatOptions::default()
        };
        let result = format("A {{.V}}", &options);

        assert!(result.diagnostics.is_empty());
    }
}
