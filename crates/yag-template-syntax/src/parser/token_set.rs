// Based on rust-analyzer's `TokenSet`.
// https://github.com/rust-lang/rust-analyzer/blob/master/crates/parser/src/token_set.rs

use crate::SyntaxKind;

#[derive(Default, Copy, Clone)]
pub(crate) struct TokenSet(u64);

const _: () = assert!(SyntaxKind::__LAST as u32 <= u64::BITS);

impl TokenSet {
    pub(crate) const fn new() -> TokenSet {
        TokenSet(0)
    }

    pub(crate) const fn add(self, kind: SyntaxKind) -> TokenSet {
        TokenSet(self.0 | mask(kind))
    }

    pub(crate) const fn union(&self, other: TokenSet) -> TokenSet {
        TokenSet(self.0 | other.0)
    }

    pub(crate) const fn contains(&self, kind: SyntaxKind) -> bool {
        (self.0 & mask(kind)) != 0
    }
}

const fn mask(kind: SyntaxKind) -> u64 {
    1 << (kind as u64)
}

pub(crate) const LEFT_DELIMS: TokenSet = TokenSet::new()
    .add(SyntaxKind::LeftDelim)
    .add(SyntaxKind::TrimmedLeftDelim);

pub(crate) const RIGHT_DELIMS: TokenSet = TokenSet::new()
    .add(SyntaxKind::RightDelim)
    .add(SyntaxKind::TrimmedRightDelim);

pub(crate) const ACTION_DELIMS: TokenSet = LEFT_DELIMS.union(RIGHT_DELIMS);
