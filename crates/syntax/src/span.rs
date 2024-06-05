use std::fmt;
use std::ops::Range;

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(lo: usize, hi: usize) -> Span {
        Span { start: lo, end: hi }
    }

    pub fn empty(pos: usize) -> Span {
        Span {
            start: pos,
            end: pos,
        }
    }

    pub fn with_start(&self, start: usize) -> Span {
        Span {
            start,
            end: self.end,
        }
    }

    pub fn with_end(&self, end: usize) -> Span {
        Span {
            start: self.start,
            end,
        }
    }
}

impl From<Range<usize>> for Span {
    fn from(r: Range<usize>) -> Span {
        Span {
            start: r.start,
            end: r.end,
        }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}
