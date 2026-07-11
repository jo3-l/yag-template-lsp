//! A formatter for YAG templates.
//!
//! This initial public surface deliberately has a conservative implementation:
//! formatting a valid template is a no-op until AST lowering is introduced in
//! later milestones. Invalid input is also returned verbatim.

#[allow(dead_code)]
mod doc;

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
/// The milestone-two formatter intentionally returns valid input unchanged.
/// It still parses every input so callers get diagnostics for malformed source
/// without ever risking an edit to it.
pub fn format(source: &str, _options: &FormatOptions) -> FormatResult {
    let parsed = yag_template_syntax::parser::parse(source);
    let diagnostics = parsed
        .errors
        .iter()
        .map(|error| FormatDiagnostic {
            kind: FormatDiagnosticKind::ParseError,
            message: error.to_string(),
        })
        .collect();
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
