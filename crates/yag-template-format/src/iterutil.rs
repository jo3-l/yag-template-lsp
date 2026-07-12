/// Iterate a slice with references to each item's immediate neighbors.
///
/// The first item has no previous neighbor and the last item has no next
/// neighbor. Empty slices yield no items.
pub(crate) fn iter_with_neighbors<T>(items: &[T]) -> impl Iterator<Item = (Option<&T>, &T, Option<&T>)> {
    let previous = std::iter::once(None).chain(items.iter().map(Some));
    let current = items.iter();
    let next = items.iter().map(Some).skip(1).chain(std::iter::once(None));

    previous
        .zip(current)
        .zip(next)
        .map(|((previous, current), next)| (previous, current, next))
}

#[cfg(test)]
mod tests {
    use super::iter_with_neighbors;

    #[test]
    fn yields_each_item_with_its_immediate_neighbors() {
        let values = ["first", "second", "third"];
        let neighbors = iter_with_neighbors(&values)
            .map(|(previous, current, next)| (previous.copied(), *current, next.copied()))
            .collect::<Vec<_>>();

        assert_eq!(
            neighbors,
            vec![
                (None, "first", Some("second")),
                (Some("first"), "second", Some("third")),
                (Some("second"), "third", None),
            ],
        );
    }

    #[test]
    fn yields_nothing_for_an_empty_slice() {
        assert_eq!(iter_with_neighbors::<u8>(&[]).count(), 0);
    }
}
