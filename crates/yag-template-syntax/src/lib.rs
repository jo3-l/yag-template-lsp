pub mod ast;
mod demo;
mod error;
mod go_lit_syntax;
pub mod lexer;
pub mod parser;
pub mod query;
mod rowan_interface;
mod syntax_kind;

pub use rowan::{TextRange, TextSize};

pub use crate::error::SyntaxError;
pub use crate::rowan_interface::{SyntaxElement, SyntaxNode, SyntaxToken, YagTemplateLanguage};
pub use crate::syntax_kind::SyntaxKind;
