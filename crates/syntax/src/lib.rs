pub mod ast;
mod ast_support;
pub mod error;
pub mod grammar;
pub mod kind;
pub mod lexer;
mod parser;
mod token_set;

use std::marker::PhantomData;

use rowan::SyntaxNodeChildren;

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

pub trait AstNode: Sized {
    fn cast(syntax: SyntaxNode) -> Option<Self>;
    fn syntax(&self) -> &SyntaxNode;
}

/// An iterator over `SyntaxNode` children of a particular AST type.
#[derive(Debug, Clone)]
pub struct AstChildren<N> {
    inner: SyntaxNodeChildren<YagTemplateLanguage>,
    ph: PhantomData<N>,
}

impl<N> AstChildren<N> {
    fn new(parent: &SyntaxNode) -> Self {
        AstChildren {
            inner: parent.children(),
            ph: PhantomData,
        }
    }
}

impl<N: AstNode> Iterator for AstChildren<N> {
    type Item = N;
    fn next(&mut self) -> Option<N> {
        self.inner.find_map(N::cast)
    }
}
