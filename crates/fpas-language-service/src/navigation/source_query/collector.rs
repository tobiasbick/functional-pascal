//! Bounded, deterministic declaration ranking shared by checked and editor discovery.

use std::{cmp::Ordering, collections::BinaryHeap};

/// The complete ordering key is independent of traversal and hash-map iteration order.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub(in crate::navigation) struct MatchKey {
    rank: u8,
    qualified: String,
    owner: String,
    file: String,
    start: usize,
    kind: String,
    spelling: String,
}

pub(in crate::navigation) struct DeclarationKey<'a> {
    pub name: &'a str,
    pub qualified: &'a str,
    pub owner: &'a str,
    pub file: &'a str,
    pub start: usize,
    pub kind: &'a str,
}

impl MatchKey {
    pub(in crate::navigation) fn new(
        query: &str,
        exact: bool,
        declaration: DeclarationKey<'_>,
    ) -> Option<Self> {
        let DeclarationKey {
            name,
            qualified,
            owner,
            file,
            start,
            kind,
        } = declaration;
        let short = name.to_ascii_lowercase();
        let canonical = qualified.to_ascii_lowercase();
        let rank = if query.is_empty() || short == query || canonical == query {
            0
        } else if exact {
            return None;
        } else if short.starts_with(query) {
            1
        } else if short.contains(query) {
            2
        } else if canonical.starts_with(query) {
            3
        } else if canonical.contains(query) {
            4
        } else {
            return None;
        };
        Some(Self {
            rank,
            qualified: canonical,
            owner: owner.into(),
            file: file.into(),
            start,
            kind: kind.into(),
            spelling: qualified.into(),
        })
    }
}

struct Entry<T, K> {
    key: K,
    value: T,
}
impl<T, K: Ord> PartialEq for Entry<T, K> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}
impl<T, K: Ord> Eq for Entry<T, K> {}
impl<T, K: Ord> PartialOrd for Entry<T, K> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<T, K: Ord> Ord for Entry<T, K> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key.cmp(&other.key)
    }
}

pub(in crate::navigation) struct Collector<T, K: Ord = MatchKey> {
    entries: BinaryHeap<Entry<T, K>>,
    limit: usize,
    total: usize,
}
impl<T, K: Ord> Collector<T, K> {
    pub(in crate::navigation) fn new(limit: usize) -> Self {
        Self {
            entries: BinaryHeap::new(),
            limit,
            total: 0,
        }
    }
    /// Constructs payloads only for retained matches; memory stays proportional to the limit.
    pub(in crate::navigation) fn offer(&mut self, key: K, value: impl FnOnce() -> T) {
        self.total += 1;
        if self.limit == 0 {
            return;
        }
        if self.entries.len() == self.limit {
            if self.entries.peek().is_some_and(|worst| key >= worst.key) {
                return;
            }
            self.entries.pop();
        }
        self.entries.push(Entry {
            key,
            value: value(),
        });
    }
    pub(in crate::navigation) fn finish(self) -> (Vec<T>, usize) {
        (
            self.entries
                .into_sorted_vec()
                .into_iter()
                .map(|entry| entry.value)
                .collect(),
            self.total,
        )
    }
}
