//! Shared bounded value presentation and immutable snapshots.

mod atomic;
/// Shared charged value and byte budgets.
pub(in crate::execution::vm) mod budget;
mod model;
/// Portable payload types and structural selectors.
pub(in crate::execution::vm) mod payload;
/// Non-executing lazy presentation.
pub(in crate::execution::vm) mod render;
/// Detached bounded value trees.
pub(in crate::execution::vm) mod snapshot;

use crate::{DebugTypeId, EnumVariantId, Value};

/// Bounds applied before enumerating children or copying strings.
#[derive(Clone, Copy)]
pub(in crate::execution::vm) struct ValueLimits {
    /// Maximum retained children per aggregate.
    pub max_children: usize,
    /// Maximum Unicode scalars per string.
    pub max_string_chars: usize,
    /// Maximum Unicode scalars in a formatted summary, including quoting.
    pub max_summary_chars: usize,
    /// Maximum number of child edges from a root value.
    pub max_depth: usize,
}

/// Structural child selector independent of debugger handles or generations.
#[derive(Clone, Debug, PartialEq)]
pub(in crate::execution::vm) enum ValuePath {
    /// Positional record field.
    RecordField(usize),
    /// Zero-based array element.
    ArrayIndex(usize),
    /// Value associated with an immutable dictionary key.
    DictionaryValue(Value),
    /// Field of the currently active enum variant.
    EnumField {
        variant: EnumVariantId,
        index: usize,
    },
    /// Active successful Result payload.
    ResultOk,
    /// Active failed Result payload.
    ResultError,
    /// Active Option payload.
    OptionSome,
}

/// Adapter for callers that retain live storage origins alongside presentation data.
pub(in crate::execution::vm) trait ValueOrigin: Clone {
    /// Origin for a child whose storage cannot be addressed.
    fn unsupported() -> Self;
    /// Carry an origin through one typed structural child.
    fn descendant(&self, component: ValuePath, expected_type: DebugTypeId) -> Self;
}

impl ValueOrigin for () {
    fn unsupported() -> Self {}
    fn descendant(&self, _: ValuePath, _: DebugTypeId) -> Self {}
}

#[cfg(test)]
mod tests;
