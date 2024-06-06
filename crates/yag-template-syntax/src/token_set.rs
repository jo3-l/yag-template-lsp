// Based on rust-analyzer's `TokenSet`.
// https://github.com/rust-lang/rust-analyzer/blob/master/crates/parser/src/token_set.rs

use crate::SyntaxKind;

#[derive(Default, Copy, Clone)]
pub(crate) struct TokenSet(u64);

const _: () = assert!(SyntaxKind::__LAST as u32 <= u64::BITS);

impl TokenSet {
    pub(crate) const EMPTY: TokenSet = TokenSet(0);

    pub(crate) const fn of(kind: SyntaxKind) -> TokenSet {
        TokenSet::EMPTY.add(kind)
    }

    pub(crate) const fn add(self, kind: SyntaxKind) -> TokenSet {
        TokenSet(self.0 | (1 << (kind as u64)))
    }

    pub(crate) const fn union(&self, other: TokenSet) -> TokenSet {
        TokenSet(self.0 | other.0)
    }

    pub(crate) const fn contains(&self, kind: SyntaxKind) -> bool {
        (self.0 >> (kind as u64) & 1) != 0
    }
}

macro_rules! token_set {
    ($($t:ident),*) => { TokenSet::EMPTY$(.add(crate::SyntaxKind::$t))* };
    ($($t:ident),* ,) => { token_set!($($t),*) };
}

pub(crate) use token_set;
