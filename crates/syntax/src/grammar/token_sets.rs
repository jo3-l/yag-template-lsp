use crate::token_set;
use crate::token_set::TokenSet;

pub(crate) const LEFT_DELIMS: TokenSet = token_set! { LeftDelim, TrimmedLeftDelim };
pub(crate) const RIGHT_DELIMS: TokenSet = token_set! { RightDelim, TrimmedRightDelim };
