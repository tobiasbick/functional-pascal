//! Explicit reservation, selected storage access and release.

use crate::{FieldId, RecordLayoutId, ValueId};

/// A synchronous reference operation on a cell root or authorized var parameter.
#[derive(Debug, Clone, PartialEq)]
pub enum ReferenceOperation {
    /// Reserve a cell root or reborrow an active reference before later argument evaluation.
    Reserve(ValueId),
    /// Select a stored field without changing the root's authority.
    Field {
        /// Reserved or active reference to the record value.
        reference: ValueId,
        /// Required record layout.
        layout: RecordLayoutId,
        /// Stored field slot.
        field: FieldId,
    },
    /// Select a collection element using an already evaluated index or key.
    Index {
        /// Reserved or active reference to the array or dictionary.
        reference: ValueId,
        /// Evaluated index or dictionary key.
        index: ValueId,
    },
    /// Read a value snapshot through the selected reference.
    Read(ValueId),
    /// Replace the selected value immediately through active authority.
    Write {
        /// Active reference to the selected storage.
        reference: ValueId,
        /// Replacement value.
        value: ValueId,
    },
    /// Release authority, including authority retained in register copies.
    Release(ValueId),
}

impl ReferenceOperation {
    /// Whether this operation defines a reference or value snapshot.
    #[must_use]
    pub const fn produces_value(&self) -> bool {
        !matches!(self, Self::Write { .. } | Self::Release(_))
    }

    /// Source values read in evaluation order by this operation.
    #[must_use]
    pub fn operands(&self) -> Vec<ValueId> {
        match self {
            Self::Reserve(value)
            | Self::Read(value)
            | Self::Release(value)
            | Self::Field {
                reference: value, ..
            } => vec![*value],
            Self::Index { reference, index } => vec![*reference, *index],
            Self::Write { reference, value } => vec![*reference, *value],
        }
    }
}
