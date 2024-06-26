use crate::SyntaxKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum YagTemplateLanguage {}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(value: SyntaxKind) -> Self {
        rowan::SyntaxKind(value as u16)
    }
}

impl rowan::Language for YagTemplateLanguage {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> SyntaxKind {
        SyntaxKind::from(raw.0)
    }

    fn kind_to_raw(kind: SyntaxKind) -> rowan::SyntaxKind {
        kind.into()
    }
}

pub type SyntaxNode = rowan::SyntaxNode<YagTemplateLanguage>;
pub type SyntaxNodePtr = rowan::ast::SyntaxNodePtr<YagTemplateLanguage>;
pub type SyntaxToken = rowan::SyntaxToken<YagTemplateLanguage>;
pub type SyntaxElement = rowan::SyntaxElement<YagTemplateLanguage>;
