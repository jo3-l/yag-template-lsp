use crate::kind::SyntaxKind;
use crate::{AstChildren, AstNode, SyntaxNode};

pub(crate) fn cast_first_child<N: AstNode>(parent: &SyntaxNode) -> Option<N> {
    parent.children().find_map(N::cast)
}

pub(crate) fn cast_children<N: AstNode>(parent: &SyntaxNode) -> AstChildren<N> {
    AstChildren::new(parent)
}

macro_rules! define_node {
    ($(#[$attr:meta])* $name:ident($pat:pat)) => {
        #[derive(Debug, Clone, Eq, PartialEq, Hash)]
        #[repr(transparent)]
        $(#[$attr])*
        pub struct $name(SyntaxNode);

        impl AstNode for $name {
            fn cast(node: SyntaxNode) -> Option<Self> {
                if matches!(node.kind(), $pat) {
                    Some(Self(node))
                } else {
                    None
                }
            }

            fn syntax(&self) -> &SyntaxNode {
                &self.0
            }
        }
    };
}

pub(crate) use define_node;
