//! Deterministic executable string table.

use crate::StringId;

/// Ordered, index-addressed UTF-8 strings used by executable metadata.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StringTable {
    entries: Vec<String>,
}

impl StringTable {
    /// Construct a table in the supplied deterministic order.
    #[must_use]
    pub fn new(entries: Vec<String>) -> Self {
        Self { entries }
    }

    /// Return a string by validated numeric identifier.
    #[must_use]
    pub fn get(&self, id: StringId) -> Option<&str> {
        usize::try_from(id.get())
            .ok()
            .and_then(|index| self.entries.get(index))
            .map(String::as_str)
    }

    /// Return the number of strings.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Return whether the table has no strings.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Iterate strings in deterministic identifier order.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &str> {
        self.entries.iter().map(String::as_str)
    }
}
