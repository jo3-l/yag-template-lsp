pub mod ast;
mod demo;
mod error;
mod go_lit_syntax;
mod kind;
pub mod lexer;
pub mod parser;
pub mod query;
mod rowan_interface;

use std::ops::Deref;

pub use rowan::{TextRange, TextSize};

pub use crate::error::SyntaxError;
pub use crate::kind::SyntaxKind;
pub use crate::rowan_interface::{SyntaxElement, SyntaxNode, SyntaxToken, YagTemplateLanguage};
