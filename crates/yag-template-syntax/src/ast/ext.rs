use std::marker::PhantomData;

use rowan::{SyntaxElementChildren, SyntaxNodeChildren};

use super::{AstNode, AstToken};
use crate::{SyntaxNode, YagTemplateLanguage};

pub trait SyntaxNodeExt {
    fn is<N: AstNode>(&self) -> bool;
    fn to<N: AstNode>(self) -> N;
    fn try_to<N: AstNode>(self) -> Option<N>;

    fn first_matching_child<N: AstNode>(&self) -> Option<N>;
    fn last_matching_child<N: AstNode>(&self) -> Option<N>;

    fn first_matching_token<T: AstToken>(&self) -> Option<T>;
    fn last_matching_token<T: AstToken>(&self) -> Option<T>;

    fn matching_children<N: AstNode>(&self) -> AstChildren<N>;
    fn matching_tokens<T: AstToken>(&self) -> AstTokenChildren<T>;
}

impl SyntaxNodeExt for SyntaxNode {
    fn is<N: AstNode>(&self) -> bool {
        N::can_cast(self.kind())
    }

    fn to<N: AstNode>(self) -> N {
        self.try_to()
            .unwrap_or_else(|| panic!("could not cast node to type {:?}", stringify!(N)))
    }

    fn try_to<N: AstNode>(self) -> Option<N> {
        N::cast(self)
    }

    fn first_matching_child<N: AstNode>(&self) -> Option<N> {
        self.children().find_map(N::cast)
    }

    fn last_matching_child<N: AstNode>(&self) -> Option<N> {
        let mut cur = self.last_child_or_token();
        while let Some(element) = cur {
            if let Some(node) = element.clone().into_node().and_then(N::cast) {
                return Some(node);
            }
            cur = element.prev_sibling_or_token();
        }
        None
    }

    fn first_matching_token<T: AstToken>(&self) -> Option<T> {
        self.children_with_tokens()
            .find_map(|element| element.into_token().and_then(T::cast))
    }

    fn last_matching_token<T: AstToken>(&self) -> Option<T> {
        let mut cursor = self.last_child_or_token();
        while let Some(element) = cursor {
            if let Some(token) = element.clone().into_token().and_then(T::cast) {
                return Some(token);
            }
            cursor = element.prev_sibling_or_token();
        }
        None
    }

    fn matching_children<N: AstNode>(&self) -> AstChildren<N> {
        AstChildren::new(self)
    }

    fn matching_tokens<T: AstToken>(&self) -> AstTokenChildren<T> {
        AstTokenChildren::new(self)
    }
}

/// An iterator over `SyntaxToken` children of a particular type.
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

/// An iterator over `SyntaxNode` children of a particular type.
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
