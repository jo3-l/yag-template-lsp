//! A formatter for YAG templates.
//!
//! Invalid input is always returned unchanged. The current lowering stages
//! format delimiter padding, parsed block indentation, and ordinary expression
//! layout. Function-specific layouts are derived from the supplied EnvDefs.

use yag_template_envdefs::EnvDefs;
use yag_template_syntax::SyntaxNode;

pub mod config;

mod cursor;
mod iterutil;
mod line_index;
mod line_protection;
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
            continuation_indent: Indent::Tabs,
            max_width: 100,
            delimiter_padding: DelimiterPadding::Spaces,
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

/// Format `source` according to `envdefs` and `options`.
pub fn format(source: &str, envdefs: &EnvDefs, options: &FormatOptions) -> FormatResult {
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
        let line_protection = line_protection::resolve(&root, source);
        let doc = lower::lower(&root, source, envdefs, options, &line_protection);
        let text = pretty::render(doc, options.max_width);
        FormatResult { text, diagnostics }
    } else {
        FormatResult {
            text: source.to_owned(),
            diagnostics,
        }
    }
}

#[cfg(test)]
mod tests {
    use yag_template_envdefs::{EnvDefSource, bundled_envdefs};

    use super::{DelimiterPadding, FormatOptions, format};

    #[test]
    fn formatting_overwidth_protected_text_has_no_diagnostics() {
        let options = FormatOptions {
            delimiter_padding: DelimiterPadding::Spaces,
            max_width: 8,
            ..FormatOptions::default()
        };
        let envdefs = bundled_envdefs::load().unwrap();
        let result = format("A {{.V}}", &envdefs, &options);

        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn formatting_adds_a_terminal_newline() {
        let envdefs = bundled_envdefs::load().unwrap();
        let result = format("{{.Value}}", &envdefs, &FormatOptions::default());

        assert_eq!(result.text, "{{ .Value }}\n");
    }

    #[test]
    fn function_layouts_are_derived_from_envdefs() {
        let envdefs = yag_template_envdefs::parse(&[EnvDefSource::new_static(
            "test.ydef",
            "func options(opts...)\nfunc keys(keyvalues...)\nfunc mixed(first, opts...)\nfunc values(args...)\nfunc optional(value)\n",
        )])
        .unwrap();
        let options = FormatOptions {
            max_width: 20,
            ..FormatOptions::default()
        };

        for name in ["options", "keys"] {
            let result = format(&format!(r#"{{{{{name} "a" "one" "b" "two"}}}}"#), &envdefs, &options);
            assert!(result.text.contains("\"a\" \"one\""), "{name}: {}", result.text);
            assert!(!result.text.contains("\"a\"\n"), "{name}: {}", result.text);
        }

        for name in ["mixed", "values", "optional"] {
            let result = format(&format!(r#"{{{{{name} "a" "one" "b" "two"}}}}"#), &envdefs, &options);
            assert!(result.text.contains("\"a\"\n"), "{name}: {}", result.text);
        }
    }
}
