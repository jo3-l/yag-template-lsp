//! Typed, document-returning formatter rules.
//!
//! Dispatch is explicit at the `Action` and `Expr` enum boundaries. Individual
//! rules return `Option<Doc>` so a malformed or unsupported typed shape can
//! fall back to its exact source without a partially written document.

mod action;
mod expr;
