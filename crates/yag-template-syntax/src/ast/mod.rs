use std::marker::PhantomData;

use rowan::SyntaxElementChildren;

use crate::{SyntaxElement, SyntaxNode, YagTemplateLanguage};

mod nodes;
mod to_typed_ext;
mod tokens;

pub use nodes::*;
pub use to_typed_ext::UntypedToTypedExt;
pub use tokens::*;

pub trait AstElement: Sized {
    fn cast(element: SyntaxElement) -> Option<Self>;
}

pub(crate) fn cast_first_child<N: AstElement>(parent: &SyntaxNode) -> Option<N> {
    parent.children_with_tokens().find_map(N::cast)
}

pub(crate) fn cast_children<N: AstElement>(parent: &SyntaxNode) -> AstElementChildren<N> {
    AstElementChildren::new(parent)
}

/// An iterator over `SyntaxElement` children of a particular AST type.
#[derive(Debug, Clone)]
pub struct AstElementChildren<N> {
    inner: SyntaxElementChildren<YagTemplateLanguage>,
    _phantom: PhantomData<N>,
}

impl<N> AstElementChildren<N> {
    pub(crate) fn new(parent: &SyntaxNode) -> Self {
        AstElementChildren {
            inner: parent.children_with_tokens(),
            _phantom: PhantomData,
        }
    }
}

impl<N: AstElement> Iterator for AstElementChildren<N> {
    type Item = N;

    fn next(&mut self) -> Option<N> {
        self.inner.find_map(N::cast)
    }
}
