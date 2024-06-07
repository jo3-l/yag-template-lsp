pub mod ast;
mod error;
mod kind;
pub mod lexer;
pub mod parser;
mod rowan_interface;

pub use rowan::{TextRange, TextSize};

pub use crate::error::SyntaxError;
pub use crate::kind::SyntaxKind;
