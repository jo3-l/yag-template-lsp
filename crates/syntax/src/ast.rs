use crate::kind::SyntaxKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum YagTemplateLanguage {}

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
pub type SyntaxToken = rowan::SyntaxToken<YagTemplateLanguage>;
pub type NodeOrToken = rowan::NodeOrToken<SyntaxNode, SyntaxToken>;
pub type SyntaxElement = rowan::SyntaxElement<YagTemplateLanguage>;
