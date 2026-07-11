use yag_template_format::{FormatDiagnosticKind, FormatOptions, FormatResult, format};
use yag_template_syntax::ast::{Action, AstNode, Expr};
use yag_template_syntax::{SyntaxElement, SyntaxKind, SyntaxNode};

/// Owned, test-only representation of the parts of a template that formatting
/// must preserve. Action trivia is excluded. Literal text preserves its
/// non-whitespace content and same-line action boundaries, while whitespace is
/// checked separately so block indentation can be formatter-owned.
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
        content: String,
        inline_prefix: String,
        inline_suffix: String,
    },
}

pub fn fingerprint(source: &str) -> TemplateFingerprint {
    let parsed = yag_template_syntax::parser::parse(source);
    assert!(parsed.errors.is_empty(), "source did not parse: {:?}", parsed.errors);
    fingerprint_node(SyntaxNode::new_root(parsed.root), source)
}

fn fingerprint_node(node: SyntaxNode, source: &str) -> TemplateFingerprint {
    let children = node.children_with_tokens().collect::<Vec<_>>();
    let children = children
        .iter()
        .enumerate()
        .filter_map(|element| match element {
            (_, SyntaxElement::Node(node)) => Some(fingerprint_node(node.clone(), source)),
            (_, SyntaxElement::Token(token)) if token.kind() == SyntaxKind::Whitespace => None,
            (index, SyntaxElement::Token(token))
                if token.kind() == SyntaxKind::Text
                    && formatter_owned_action_separator(&children, index, token.text()) =>
            {
                None
            }
            (_, SyntaxElement::Token(token)) if token.kind() == SyntaxKind::Text => {
                let layout = literal_text_layout(token.text(), token_start(token), token_end(token), source);
                (!layout.content.is_empty() || !layout.inline_prefix.is_empty() || !layout.inline_suffix.is_empty())
                    .then_some(TemplateFingerprint::Text {
                        content: layout.content,
                        inline_prefix: layout.inline_prefix,
                        inline_suffix: layout.inline_suffix,
                    })
            }
            (_, SyntaxElement::Token(token)) => Some(TemplateFingerprint::Token {
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

/// Whitespace directly between two non-display actions is formatter-owned.
/// Display actions remain excluded because their surrounding literal spacing
/// is output-facing.
fn formatter_owned_action_separator(children: &[SyntaxElement], index: usize, text: &str) -> bool {
    text.chars().all(char::is_whitespace)
        && index > 0
        && index + 1 < children.len()
        && children[index - 1]
            .clone()
            .into_node()
            .and_then(Action::cast)
            .is_some_and(|action| !is_display_action(action))
        && children[index + 1]
            .clone()
            .into_node()
            .and_then(Action::cast)
            .is_some_and(|action| !is_display_action(action))
}

fn is_display_action(action: Action) -> bool {
    matches!(action, Action::ExprAction(action) if action.expr().is_some_and(qualifies_display_expr))
}

fn qualifies_display_expr(expr: Expr) -> bool {
    match expr {
        Expr::VarAccess(_) | Expr::ContextAccess(_) | Expr::ContextFieldChain(_) => true,
        Expr::ExprFieldChain(chain) => chain.base_expr().is_some_and(qualifies_display_expr),
        Expr::Parenthesized(parenthesized) => parenthesized.inner_expr().is_some_and(qualifies_display_expr),
        _ => false,
    }
}

/// Whether formatting changed whitespace inside literal content, rather than
/// only adding/removing indentation at the physical line margins. Such changes
/// remain permitted, but later lowering must report `LiteralWhitespaceChanged`.
pub fn has_internal_literal_whitespace_change(before: &str, after: &str) -> bool {
    let before = literal_text_layouts(before);
    let after = literal_text_layouts(after);
    before.len() == after.len()
        && before.iter().zip(after).any(|(before, after)| {
            before.content == after.content
                && before.inline_prefix == after.inline_prefix
                && before.inline_suffix == after.inline_suffix
                && before.margin_normalized != after.margin_normalized
        })
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct LiteralTextLayout {
    content: String,
    inline_prefix: String,
    inline_suffix: String,
    margin_normalized: String,
}

fn literal_text_layouts(source: &str) -> Vec<LiteralTextLayout> {
    let parsed = yag_template_syntax::parser::parse(source);
    assert!(parsed.errors.is_empty(), "source did not parse: {:?}", parsed.errors);
    let mut layouts = Vec::new();
    collect_literal_text_layouts(SyntaxNode::new_root(parsed.root), source, &mut layouts);
    layouts
}

fn collect_literal_text_layouts(node: SyntaxNode, source: &str, layouts: &mut Vec<LiteralTextLayout>) {
    for element in node.children_with_tokens() {
        match element {
            SyntaxElement::Node(node) => collect_literal_text_layouts(node, source, layouts),
            SyntaxElement::Token(token) if token.kind() == SyntaxKind::Text => {
                layouts.push(literal_text_layout(
                    token.text(),
                    token_start(&token),
                    token_end(&token),
                    source,
                ));
            }
            SyntaxElement::Token(_) => {}
        }
    }
}

fn literal_text_layout(text: &str, start: usize, end: usize, source: &str) -> LiteralTextLayout {
    let starts_line = start == 0 || source.as_bytes()[start - 1] == b'\n';
    let ends_line = end == source.len() || source.as_bytes()[end] == b'\n';
    let leading = leading_whitespace(text);
    let trailing = trailing_whitespace(text);
    LiteralTextLayout {
        content: text.chars().filter(|character| !character.is_whitespace()).collect(),
        inline_prefix: if !starts_line && !leading.contains('\n') {
            leading.to_owned()
        } else {
            String::new()
        },
        inline_suffix: if !ends_line && !trailing.contains('\n') {
            trailing.to_owned()
        } else {
            String::new()
        },
        margin_normalized: trim_line_margins(text, starts_line, ends_line),
    }
}

fn leading_whitespace(text: &str) -> &str {
    let end = text
        .find(|character: char| !character.is_whitespace())
        .unwrap_or(text.len());
    &text[..end]
}

fn trailing_whitespace(text: &str) -> &str {
    let start = text
        .char_indices()
        .rev()
        .find(|(_, character)| !character.is_whitespace())
        .map_or(0, |(offset, character)| offset + character.len_utf8());
    &text[start..]
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

/// The preservation contract every valid formatter fixture uses from this
/// milestone onward.
#[allow(dead_code)]
pub fn assert_formats_preserving_fingerprint(source: &str, options: &FormatOptions) {
    let formatted = format(source, options);
    assert_format_result_preserving_fingerprint(source, options, &formatted);
}

pub fn assert_format_result_preserving_fingerprint(source: &str, options: &FormatOptions, formatted: &FormatResult) {
    let input_fingerprint = fingerprint(source);
    assert!(
        formatted
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.kind != FormatDiagnosticKind::ParseError),
        "formatter reported a parse error: {:?}",
        formatted.diagnostics
    );
    let output_fingerprint = fingerprint(&formatted.text);
    assert_eq!(
        output_fingerprint, input_fingerprint,
        "formatter changed semantic template shape"
    );
    if has_internal_literal_whitespace_change(source, &formatted.text) {
        assert!(
            formatted
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.kind == FormatDiagnosticKind::LiteralWhitespaceChanged),
            "formatter changed internal literal whitespace without a diagnostic: {:?}",
            formatted.diagnostics
        );
    }
    assert_eq!(
        format(&formatted.text, options).text,
        formatted.text,
        "formatter is not idempotent"
    );
}
