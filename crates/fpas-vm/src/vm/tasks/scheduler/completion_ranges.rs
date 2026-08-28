//! Compact membership storage for consumed monotone task identifiers.

use std::collections::BTreeMap;
use std::ops::Bound::{Excluded, Unbounded};

/// Disjoint inclusive ranges of consumed task identifiers.
#[derive(Default)]
pub(super) struct CompletionRanges {
    ranges: BTreeMap<u64, u64>,
}

impl CompletionRanges {
    /// Returns whether `id` belongs to a consumed range.
    pub(super) fn contains(&self, id: &u64) -> bool {
        self.ranges
            .range(..=id)
            .next_back()
            .is_some_and(|(_, end)| id <= end)
    }

    /// Inserts `id` and merges directly adjacent ranges.
    pub(super) fn insert(&mut self, id: u64) {
        if self.contains(&id) {
            return;
        }

        let predecessor = self
            .ranges
            .range(..id)
            .next_back()
            .map(|(&start, &end)| (start, end));
        let successor = self
            .ranges
            .range((Excluded(id), Unbounded))
            .next()
            .map(|(&start, &end)| (start, end));
        let joins_predecessor = predecessor.is_some_and(|(_, end)| end.checked_add(1) == Some(id));
        let joins_successor = successor.is_some_and(|(start, _)| id.checked_add(1) == Some(start));

        match (predecessor, successor, joins_predecessor, joins_successor) {
            (Some((start, _)), Some((next_start, next_end)), true, true) => {
                self.ranges.insert(start, next_end);
                self.ranges.remove(&next_start);
            }
            (Some((start, _)), _, true, false) => {
                self.ranges.insert(start, id);
            }
            (_, Some((next_start, next_end)), false, true) => {
                self.ranges.remove(&next_start);
                self.ranges.insert(id, next_end);
            }
            _ => {
                self.ranges.insert(id, id);
            }
        }
    }

    #[cfg(test)]
    pub(super) fn range_count(&self) -> usize {
        self.ranges.len()
    }
}

#[cfg(test)]
mod tests {
    use super::CompletionRanges;

    #[test]
    fn bridging_id_merges_two_ranges() {
        let mut completions = CompletionRanges::default();
        completions.insert(2);
        completions.insert(4);

        completions.insert(3);

        assert_eq!(
            (
                completions.range_count(),
                completions.contains(&2),
                completions.contains(&3),
                completions.contains(&4)
            ),
            (1, true, true, true)
        );
    }

    #[test]
    fn gap_remains_unknown() {
        let mut completions = CompletionRanges::default();
        completions.insert(1);
        completions.insert(3);

        assert_eq!(
            (completions.range_count(), completions.contains(&2)),
            (2, false)
        );
    }

    #[test]
    fn maximum_identifier_merges_without_overflow() {
        let mut completions = CompletionRanges::default();
        completions.insert(u64::MAX);
        completions.insert(u64::MAX - 1);

        assert_eq!(
            (completions.range_count(), completions.contains(&u64::MAX)),
            (1, true)
        );
    }
}
