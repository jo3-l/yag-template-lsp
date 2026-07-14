use std::fmt::Display;

use yag_template_format::{FormatDiagnosticKind, FormatOptions, FormatResult, format};
use yag_template_syntax::{SyntaxElement, SyntaxKind, SyntaxNode};

/// Owned, test-only representation of the parts of a template that formatting
/// must preserve. Action trivia and whitespace-only literal spans are excluded
/// because the formatter owns their layout. Literal text with content retains
/// all whitespace except indentation at physical line margins.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TemplateFingerprint {
    Node {
        kind: SyntaxKind,
        children: Vec<TemplateFingerprint>,
    },
    Token {
        kind: SyntaxKind,
        text: String,
    },
    Text {
        text: String,
    },
}

pub fn fingerprint(source: &str) -> TemplateFingerprint {
    let parsed = yag_template_syntax::parser::parse(source);
    assert!(parsed.errors.is_empty(), "source did not parse: {:?}", parsed.errors);
    fingerprint_node(SyntaxNode::new_root(parsed.root), source)
}

fn fingerprint_node(node: SyntaxNode, source: &str) -> TemplateFingerprint {
    let children = node
        .children_with_tokens()
        .filter_map(|element| match element {
            SyntaxElement::Node(node) => Some(fingerprint_node(node, source)),
            SyntaxElement::Token(token) if token.kind() == SyntaxKind::Whitespace => None,
            SyntaxElement::Token(token)
                if token.kind() == SyntaxKind::Text && token.text().chars().all(char::is_whitespace) =>
            {
                None
            }
            SyntaxElement::Token(token) if token.kind() == SyntaxKind::Text => Some(TemplateFingerprint::Text {
                text: normalize_literal_text(token.text(), token_start(&token), token_end(&token), source),
            }),
            SyntaxElement::Token(token) => Some(TemplateFingerprint::Token {
                kind: token.kind(),
                text: match token.kind() {
                    SyntaxKind::TrimmedLeftDelim => "{{-".to_owned(),
                    SyntaxKind::TrimmedRightDelim => "-}}".to_owned(),
                    _ => token.text().to_owned(),
                },
            }),
        })
        .collect();
    TemplateFingerprint::Node {
        kind: node.kind(),
        children,
    }
}

fn normalize_literal_text(text: &str, start: usize, end: usize, source: &str) -> String {
    let starts_line = start == 0 || source.as_bytes()[start - 1] == b'\n';
    let ends_line = end == source.len() || source.as_bytes()[end] == b'\n';
    let normalized = trim_line_margins(text, starts_line, ends_line);
    if end == source.len() {
        normalized.strip_suffix('\n').unwrap_or(&normalized).to_owned()
    } else {
        normalized
    }
}

fn trim_line_margins(text: &str, mut starts_line: bool, ends_line: bool) -> String {
    let mut normalized = String::new();
    for segment in text.split_inclusive('\n') {
        let has_newline = segment.ends_with('\n');
        let mut content = segment.strip_suffix('\n').unwrap_or(segment);
        if starts_line {
            content = content.trim_start_matches(char::is_whitespace);
        }
        if has_newline || ends_line {
            content = content.trim_end_matches(char::is_whitespace);
        }
        normalized.push_str(content);
        if has_newline {
            normalized.push('\n');
        }
        starts_line = true;
    }
    normalized
}

fn token_start(token: &yag_template_syntax::SyntaxToken) -> usize {
    byte_offset(token.text_range().start())
}

fn token_end(token: &yag_template_syntax::SyntaxToken) -> usize {
    byte_offset(token.text_range().end())
}

fn byte_offset(position: impl Into<u32>) -> usize {
    position.into() as usize
}

#[allow(dead_code)]
pub fn assert_format_result_preserving_fingerprint(
    source: &str,
    options: &FormatOptions,
    formatted: &FormatResult,
    context: impl Display,
) {
    let input_fingerprint = fingerprint(source);
    assert!(
        formatted
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.kind != FormatDiagnosticKind::ParseError),
        "{context}: formatter reported a parse error: {:?}",
        formatted.diagnostics
    );
    let output_fingerprint = fingerprint(&formatted.text);
    assert_eq!(
        output_fingerprint, input_fingerprint,
        "{context}: formatter changed semantic template shape"
    );
    assert_eq!(
        format(&formatted.text, options).text,
        formatted.text,
        "{context}: formatter is not idempotent"
    );
}
