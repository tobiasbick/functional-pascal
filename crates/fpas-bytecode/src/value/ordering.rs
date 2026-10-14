//! Scalar ordering shared by comparisons and stable collection sorting.
//! See `docs/pascal/language/types/array/basics.md`.

use super::Value;
use std::cmp::Ordering;

impl Value {
    /// Compares compatible runtime scalar values; aggregates and mixed types have no order.
    ///
    /// Real signed zeros compare equal. Callers validate finite-real requirements
    /// before using this order. Subranges, distinct scalars, and payload-free enums
    /// use their underlying scalar representation after lowering.
    #[must_use]
    pub fn scalar_ordering(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Self::Integer(left), Self::Integer(right)) => Some(left.cmp(right)),
            (Self::Boolean(left), Self::Boolean(right)) => Some(left.cmp(right)),
            (Self::Str(left), Self::Str(right)) => Some(left.cmp(right)),
            (Self::Real(left), Self::Real(right)) => left.partial_cmp(right),
            _ => None,
        }
    }
}
