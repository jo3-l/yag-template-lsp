use rowan::{SyntaxElementChildren, TextRange};

use crate::{SyntaxKind, SyntaxNode, SyntaxToken};

pub mod ext;
mod macros;
mod nodes;
mod tokens;

pub use nodes::*;
pub use tokens::*;

pub trait AstNode {
    fn can_cast(kind: SyntaxKind) -> bool;

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &SyntaxNode;

    fn text_range(&self) -> TextRange {
        self.syntax().text_range()
    }
}

pub trait AstToken {
    fn can_cast(kind: SyntaxKind) -> bool;

    fn cast(syntax: SyntaxToken) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &SyntaxToken;

    fn text_range(&self) -> TextRange {
        self.syntax().text_range()
    }
}
