use std::marker::PhantomData;

use rowan::{SyntaxElementChildren, SyntaxNodeChildren};

use crate::{SyntaxNode, SyntaxToken, YagTemplateLanguage};

mod nodes;
mod tokens;

pub use nodes::*;
pub use tokens::*;

pub trait AstNode: Sized {
    fn cast(syntax: SyntaxNode) -> Option<Self>;
    fn syntax(&self) -> &SyntaxNode;
}

pub trait AstToken: Sized {
    fn cast(syntax: SyntaxToken) -> Option<Self>;
    fn syntax(&self) -> &SyntaxToken;
}

pub trait SyntaxNodeExt: Sized + Clone {
    fn is<N: AstNode>(&self) -> bool {
        self.clone().try_to::<N>().is_some()
    }

    fn to<N: AstNode>(self) -> N {
        self.try_to().unwrap_or_else(|| {
            panic!("failed to cast node as `{:?}`", stringify!(T));
        })
    }

    fn try_to<N: AstNode>(self) -> Option<N>;

    fn find_first_child<N: AstNode>(&self) -> Option<N>;
    fn find_last_child<N: AstNode>(&self) -> Option<N>;

    fn find_first_token<T: AstToken>(&self) -> Option<T>;
    fn find_last_token<T: AstToken>(&self) -> Option<T>;

    fn cast_children<N: AstNode>(&self) -> AstChildren<N>;
    fn cast_tokens<T: AstToken>(&self) -> AstTokenChildren<T>;
}

impl SyntaxNodeExt for SyntaxNode {
    fn try_to<N: AstNode>(self) -> Option<N> {
        N::cast(self)
    }

    fn find_first_child<N: AstNode>(&self) -> Option<N> {
        self.children().find_map(N::cast)
    }

    fn find_last_child<N: AstNode>(&self) -> Option<N> {
        let mut cur = self.last_child_or_token();
        while let Some(element) = cur {
            if let Some(node) = element.clone().into_node().and_then(N::cast) {
                return Some(node);
            }
            cur = element.prev_sibling_or_token();
        }
        None
    }

    fn find_first_token<T: AstToken>(&self) -> Option<T> {
        self.children_with_tokens()
            .find_map(|element| element.into_token().and_then(T::cast))
    }

    fn find_last_token<T: AstToken>(&self) -> Option<T> {
        let mut cursor = self.last_child_or_token();
        while let Some(element) = cursor {
            if let Some(token) = element.clone().into_token().and_then(T::cast) {
                return Some(token);
            }
            cursor = element.prev_sibling_or_token();
        }
        None
    }

    fn cast_children<N: AstNode>(&self) -> AstChildren<N> {
        AstChildren::new(self)
    }

    fn cast_tokens<T: AstToken>(&self) -> AstTokenChildren<T> {
        AstTokenChildren::new(self)
    }
}

/// An iterator over `SyntaxToken` children of a particular AST type.
#[derive(Debug, Clone)]
pub struct AstTokenChildren<T: AstToken> {
    inner: SyntaxElementChildren<YagTemplateLanguage>,
    _phantom: PhantomData<T>,
}

impl<T: AstToken> AstTokenChildren<T> {
    pub(crate) fn new(parent: &SyntaxNode) -> Self {
        Self {
            inner: parent.children_with_tokens(),
            _phantom: PhantomData,
        }
    }
}

impl<T: AstToken> Iterator for AstTokenChildren<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.find_map(|element| element.into_token().and_then(T::cast))
    }
}

/// An iterator over `SyntaxNode` children of a particular AST type.
#[derive(Debug, Clone)]
pub struct AstChildren<T: AstNode> {
    inner: SyntaxNodeChildren<YagTemplateLanguage>,
    _phantom: PhantomData<T>,
}

impl<T: AstNode> AstChildren<T> {
    pub(crate) fn new(parent: &SyntaxNode) -> Self {
        AstChildren {
            inner: parent.children(),
            _phantom: PhantomData,
        }
    }
}

impl<T: AstNode> Iterator for AstChildren<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.inner.find_map(T::cast)
    }
}
