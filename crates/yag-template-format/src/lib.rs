//! A formatter for YAG templates.
//!
//! This public surface deliberately remains conservative: valid templates are
//! unchanged until AST lowering is introduced in later milestones. The region
//! classifier may still report protected lines that exceed the configured
//! width. Invalid input is always returned verbatim.

use yag_template_syntax::SyntaxNode;

#[allow(dead_code)]
mod doc;
mod line_index;
mod region;

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

/// Formatter configuration.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FormatOptions {
    pub indent: Indent,
    pub continuation_indent: Indent,
    pub max_width: usize,
    pub delimiter_padding: DelimiterPadding,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            indent: Indent::Tabs,
            continuation_indent: Indent::Spaces(2),
            max_width: 100,
            delimiter_padding: DelimiterPadding::None,
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
/// The formatter intentionally returns valid input unchanged until AST
/// lowering is introduced. It still parses every input and classifies valid
/// source so callers receive parse and protected-over-width diagnostics without
/// risking an edit.
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
        let protected_textual_lines =
            region::protected_textual_line_mask(&SyntaxNode::new_root(parsed.root.clone()), source);
        diagnostics.extend(
            source
                .split('\n')
                .enumerate()
                .filter(|(line, text)| {
                    protected_textual_lines[*line]
                        && text.strip_suffix('\r').unwrap_or(text).chars().count() > options.max_width
                })
                .map(|(line, _)| FormatDiagnostic {
                    kind: FormatDiagnosticKind::ProtectedOverWidthLine,
                    message: format!(
                        "protected-textual template region on line {} exceeds the configured width",
                        line + 1
                    ),
                }),
        );
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
                continuation_indent: Indent::Spaces(2),
                max_width: 100,
                delimiter_padding: DelimiterPadding::None,
            }
        );
    }

    #[test]
    fn valid_source_is_unchanged() {
        let source = "{{  if .Enabled }}\n{{ .Name }}\n{{ end }}";
        let result = format(source, &FormatOptions::default());
        assert_eq!(result.text, source);
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
