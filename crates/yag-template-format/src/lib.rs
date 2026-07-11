//! A formatter for YAG templates.
//!
//! Invalid input is always returned verbatim. The current lowering stage
//! formats only ordinary delimiter padding; blocks and expressions retain
//! their source layout until their dedicated formatter stages.

use yag_template_syntax::SyntaxNode;

#[allow(dead_code)] // Expression and block lowering use the remaining variants in later milestones.
mod doc;
mod line_index;
mod lower;
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
        let line_plan = region::classify(&root, source);
        let protected_textual_lines = line_plan.protected_textual_line_mask();
        let text = doc::render(lower::lower(&root, source, options, &line_plan), options.max_width);
        diagnostics.extend(
            text.split('\n')
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
                continuation_indent: Indent::Spaces(2),
                max_width: 100,
                delimiter_padding: DelimiterPadding::None,
            }
        );
    }

    #[test]
    fn valid_source_is_lowered_conservatively() {
        let source = "{{  if .Enabled }}\n{{ .Name }}\n{{ end }}";
        let result = format(source, &FormatOptions::default());
        assert_eq!(result.text, "{{if .Enabled}}\n{{.Name}}\n{{end}}");
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
