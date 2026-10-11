//! Structural integer subrange types.
//!
//! **Documentation:** `docs/pascal/language/types/subranges.md`

/// Inclusive integer bounds and the spelling used in diagnostics.
#[derive(Debug, Clone)]
pub struct SubrangeTy {
    /// Source-visible type name; not part of type identity.
    pub name: String,
    /// Inclusive minimum.
    pub lower: i64,
    /// Inclusive maximum.
    pub upper: i64,
}

impl PartialEq for SubrangeTy {
    fn eq(&self, other: &Self) -> bool {
        self.lower == other.lower && self.upper == other.upper
    }
}

impl SubrangeTy {
    /// Whether the integer lies inside both inclusive endpoints.
    pub fn contains(&self, value: i64) -> bool {
        (self.lower..=self.upper).contains(&value)
    }
}
