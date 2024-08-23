macro_rules! define_ast_node {
    ($(#[$attr:meta])* pub struct $name:ident;) => {
        #[derive(Debug, Clone, Eq, PartialEq, Hash)]
        #[repr(transparent)]
        $(#[$attr])*
        pub struct $name {
            syntax: crate::SyntaxNode,
        }

        impl crate::ast::AstNode for $name {
            fn can_cast(kind: crate::SyntaxKind) -> bool {
                kind == crate::SyntaxKind::$name
            }

            fn cast(syntax: crate::SyntaxNode) -> Option<Self> {
                if Self::can_cast(syntax.kind()) {
                    Some(Self { syntax })
                } else {
                    None
                }
            }

            fn syntax(&self) -> &crate::SyntaxNode {
                &self.syntax
            }
        }
    }
}

pub(crate) use define_ast_node;

macro_rules! define_delim_accessors {
    ($name:ident) => {
        impl $name {
            pub fn left_delim(&self) -> Option<tokens::LeftDelim> {
                self.syntax.last_matching_token()
            }

            pub fn right_delim(&self) -> Option<tokens::RightDelim> {
                self.syntax.first_matching_token()
            }
        }
    };
}

pub(crate) use define_delim_accessors;

macro_rules! define_ast_enum {
    (
        $(#[$attr:meta])*
        pub enum $name:ident {
            $($(#[$varattr:meta])* $varname:ident($varty:ty),)*
        }
    ) => {
        #[derive(Debug, Clone, Eq, PartialEq, Hash)]
        $(#[$attr])*
        pub enum $name {
            $($(#[$varattr])* $varname($varty),)*
        }

        impl crate::ast::AstNode for $name {
            fn can_cast(kind: crate::SyntaxKind) -> bool {
                $(<$varty>::can_cast(kind) ||)* false
            }

            fn cast(syntax: crate::SyntaxNode) -> Option<Self> {
                $(
                    if <$varty>::can_cast(syntax.kind()) {
                        return <$varty>::cast(syntax).map(Self::$varname);
                    }
                )*
                return None;
            }

            fn syntax(&self) -> &crate::SyntaxNode {
                match self {
                    $(Self::$varname(v) => v.syntax(),)*
                }
            }
        }
    };
}

pub(crate) use define_ast_enum;

macro_rules! define_ast_token {
    ($(#[$attr:meta])* pub struct $name:ident;) => {
        #[derive(Debug, Clone, Eq, PartialEq, Hash)]
        #[repr(transparent)]
        $(#[$attr])*
        pub struct $name {
            syntax: crate::SyntaxToken,
        }

        impl crate::ast::AstToken for $name {
            fn can_cast(kind: crate::SyntaxKind) -> bool {
                kind == SyntaxKind::$name
            }

            fn cast(syntax: crate::SyntaxToken) -> Option<Self> {
                if Self::can_cast(syntax.kind()) {
                    Some(Self { syntax })
                } else {
                    None
                }
            }

            fn syntax(&self) -> &crate::SyntaxToken {
                &self.syntax
            }
        }
    }
}

pub(crate) use define_ast_token;
