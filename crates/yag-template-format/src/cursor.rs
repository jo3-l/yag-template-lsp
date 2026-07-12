//! Small slice cursors used while lowering formatter structures.

/// A generic double-ended peek cursor over a slice.
///
/// It supports inspecting either edge and its adjacent item before advancing,
/// while retaining the unconsumed slice for callers that need it.
pub(crate) struct DoubleEndedPeekable<'a, T> {
    elements: &'a [T],
    start: usize,
    end: usize,
}

impl<'a, T> DoubleEndedPeekable<'a, T> {
    pub(crate) fn new(elements: &'a [T]) -> Self {
        Self {
            elements,
            start: 0,
            end: elements.len(),
        }
    }

    pub(crate) fn peek_first(&self) -> Option<&'a T> {
        self.remaining().first()
    }

    pub(crate) fn peek_second(&self) -> Option<&'a T> {
        self.remaining().get(1)
    }

    pub(crate) fn peek_last(&self) -> Option<&'a T> {
        self.remaining().last()
    }

    pub(crate) fn peek_secondlast(&self) -> Option<&'a T> {
        self.remaining().get(self.remaining().len().checked_sub(2)?)
    }

    pub(crate) fn drop_first(&mut self) {
        debug_assert!(self.start < self.end);
        self.start += 1;
    }

    pub(crate) fn drop_last(&mut self) {
        debug_assert!(self.start < self.end);
        self.end -= 1;
    }

    pub(crate) fn remaining(&self) -> &'a [T] {
        &self.elements[self.start..self.end]
    }
}

#[cfg(test)]
mod tests {
    use super::DoubleEndedPeekable;

    #[test]
    fn peeks_and_advances_from_both_ends() {
        let values = [1, 2, 3, 4];
        let mut cursor = DoubleEndedPeekable::new(&values);

        assert_eq!(cursor.peek_first(), Some(&1));
        assert_eq!(cursor.peek_second(), Some(&2));
        assert_eq!(cursor.peek_last(), Some(&4));
        assert_eq!(cursor.peek_secondlast(), Some(&3));

        cursor.drop_first();
        cursor.drop_last();

        assert_eq!(cursor.remaining(), [2, 3]);
        assert_eq!(cursor.peek_first(), Some(&2));
        assert_eq!(cursor.peek_last(), Some(&3));
    }
}
