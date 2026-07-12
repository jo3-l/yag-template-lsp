//! A formatter for YAG templates.
//!
//! Invalid input is always returned unchanged. The current lowering stages
//! format delimiter padding, parsed block indentation, and ordinary expression
//! layout; specialized function layouts remain for later stages.

use std::collections::{BTreeMap, BTreeSet};

use yag_template_syntax::SyntaxNode;

mod classification;
mod doc;
mod line_index;
mod lower;
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
    pub by_name: BTreeMap<String, LayoutKind>,
}

impl Default for FunctionLayouts {
    fn default() -> Self {
        Self {
            by_name: BTreeMap::from([
                (
                    "dict".to_owned(),
                    LayoutKind::KeyValuePairs {
                        dangling_value: DanglingValuePolicy::PreserveCallLayout,
                    },
                ),
                (
                    "sdict".to_owned(),
                    LayoutKind::KeyValuePairs {
                        dangling_value: DanglingValuePolicy::PreserveCallLayout,
                    },
                ),
            ]),
        }
    }
}

/// A configured function's document layout.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum LayoutKind {
    Call,
    KeyValuePairs { dangling_value: DanglingValuePolicy },
}

/// How a key/value layout handles an unmatched final argument.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DanglingValuePolicy {
    PreserveCallLayout,
    Error,
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
    ProtectedOverWidthLine,
    /// Formatting changed non-margin whitespace inside literal template text.
    /// The output remains valid, but the change may alter rendered content.
    LiteralWhitespaceChanged,
    OddKeyValueArgumentCount,
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
    let mut diagnostics = parsed
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
        let protected_textual_lines = line_plan.protected_textual_line_mask();
        let (doc, lowering_diagnostics) = lower::lower(&root, source, options, &line_plan);
        diagnostics.extend(lowering_diagnostics);
        let text = doc::render(doc, options.max_width);
        // A source check preserves diagnostics for protected lines after an
        // earlier flexible expression adds output lines. The unbounded render
        // also catches delimiter padding that makes an otherwise fitting line
        // exceed the limit.
        let (protected_doc, _) = lower::lower(&root, source, options, &line_plan);
        let protected_line_text = doc::render(protected_doc, usize::MAX);
        let mut protected_over_width = source
            .split('\n')
            .enumerate()
            .filter(|(line, text)| {
                protected_textual_lines.get(*line).copied().unwrap_or(false)
                    && text.strip_suffix('\r').unwrap_or(text).chars().count() > options.max_width
            })
            .map(|(line, _)| line)
            .collect::<BTreeSet<_>>();
        protected_over_width.extend(
            protected_line_text
                .split('\n')
                .enumerate()
                .filter(|(line, text)| {
                    protected_textual_lines.get(*line).copied().unwrap_or(false)
                        && text.strip_suffix('\r').unwrap_or(text).chars().count() > options.max_width
                })
                .map(|(line, _)| line),
        );
        diagnostics.extend(protected_over_width.into_iter().map(|line| FormatDiagnostic {
            kind: FormatDiagnosticKind::ProtectedOverWidthLine,
            message: format!(
                "protected-textual template region on line {} exceeds the configured width",
                line + 1
            ),
        }));
        return FormatResult { text, diagnostics };
    }
    FormatResult {
        text: source.to_owned(),
        diagnostics,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_the_formatter_contract() {
        assert_eq!(
            FormatOptions::default(),
            FormatOptions {
                indent: Indent::Tabs,
                continuation_indent: Indent::Tabs,
                max_width: 100,
                delimiter_padding: DelimiterPadding::Spaces,
                function_layouts: FunctionLayouts::default(),
            }
        );
    }

    #[test]
    fn valid_source_is_lowered_conservatively() {
        let source = "{{  if .Enabled }}\n{{ .Name }}\n{{ end }}";
        let result = format(source, &FormatOptions::default());
        assert_eq!(result.text, "{{ if .Enabled }}\n\t{{ .Name }}\n{{ end }}");
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn malformed_source_is_unchanged_with_diagnostics() {
        let source = "{{ if";
        let result = format(source, &FormatOptions::default());
        assert_eq!(result.text, source);
        assert!(matches!(
            result.diagnostics.first().map(|d| &d.kind),
            Some(FormatDiagnosticKind::ParseError)
        ));
    }
}
