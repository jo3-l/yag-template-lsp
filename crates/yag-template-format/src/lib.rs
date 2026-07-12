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
    use super::{DelimiterPadding, FormatDiagnosticKind, FormatOptions, format};

    #[test]
    fn protected_width_diagnostics_measure_the_formatted_action() {
        let options = FormatOptions {
            delimiter_padding: DelimiterPadding::Spaces,
            max_width: 8,
            ..FormatOptions::default()
        };
        let result = format("A {{.V}}", &options);

        assert!(
            result
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.kind == FormatDiagnosticKind::ProtectedOverWidthLine)
        );
    }

    #[test]
    fn protected_width_diagnostics_survive_earlier_flexible_wrapping() {
        let options = FormatOptions {
            max_width: 12,
            ..FormatOptions::default()
        };
        let result = format("{{print .A .B .C}}\nHello {{ .Very.Long.Field }}!", &options);

        assert!(
            result
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.kind == FormatDiagnosticKind::ProtectedOverWidthLine)
        );
    }

    #[test]
    fn odd_key_value_arguments_report_a_diagnostic() {
        let options = FormatOptions {
            max_width: 14,
            ..FormatOptions::default()
        };
        let result = format("{{sdict \"a\" \"one\" \"dangling\"}}", &options);

        assert!(
            result
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.kind == FormatDiagnosticKind::OddKeyValueArgumentCount)
        );
    }

    #[test]
    fn protected_textual_overwidth_is_reported_without_reflowing() {
        let source = "Hello, {{ .User.Username }}! This literal line is intentionally too long.";
        let options = FormatOptions {
            max_width: 20,
            ..FormatOptions::default()
        };
        let result = format(source, &options);

        assert_eq!(result.text, source);
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.kind == FormatDiagnosticKind::ProtectedOverWidthLine)
        );
    }

    #[test]
    fn protected_crlf_line_width_excludes_the_line_terminator() {
        let options = FormatOptions {
            max_width: 10,
            ..FormatOptions::default()
        };
        let result = format("A {{.V}}\r\n", &options);

        assert!(
            result
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.kind != FormatDiagnosticKind::ProtectedOverWidthLine)
        );
    }
}
