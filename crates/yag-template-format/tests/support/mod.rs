use yag_template_format::{FormatOptions, format};
use yag_template_syntax::{SyntaxElement, SyntaxKind, SyntaxNode};

/// Owned, test-only representation of the parts of a template that formatting
/// must preserve. Trivia inside actions and whitespace-only literal spans are
/// excluded because they are formatter-owned padding or structural breaks.
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
}

pub fn fingerprint(source: &str) -> TemplateFingerprint {
    let parsed = yag_template_syntax::parser::parse(source);
    assert!(parsed.errors.is_empty(), "source did not parse: {:?}", parsed.errors);
    fingerprint_node(SyntaxNode::new_root(parsed.root))
}

fn fingerprint_node(node: SyntaxNode) -> TemplateFingerprint {
    let children = node
        .children_with_tokens()
        .filter_map(|element| match element {
            SyntaxElement::Node(node) => Some(fingerprint_node(node)),
            SyntaxElement::Token(token) if token.kind() == SyntaxKind::Whitespace => None,
            SyntaxElement::Token(token)
                if token.kind() == SyntaxKind::Text && token.text().chars().all(char::is_whitespace) =>
            {
                None
            }
            SyntaxElement::Token(token) => Some(TemplateFingerprint::Token {
                kind: token.kind(),
                text: token.text().to_owned(),
            }),
        })
        .collect();
    TemplateFingerprint::Node {
        kind: node.kind(),
        children,
    }
}

/// The preservation contract every valid formatter fixture uses from this
/// milestone onward.
pub fn assert_formats_preserving_fingerprint(source: &str, options: &FormatOptions) {
    let input_fingerprint = fingerprint(source);
    let formatted = format(source, options);
    assert!(
        formatted.diagnostics.is_empty(),
        "format diagnostics: {:?}",
        formatted.diagnostics
    );
    let output_fingerprint = fingerprint(&formatted.text);
    assert_eq!(
        output_fingerprint, input_fingerprint,
        "formatter changed semantic template shape"
    );
    assert_eq!(
        format(&formatted.text, options).text,
        formatted.text,
        "formatter is not idempotent"
    );
}
