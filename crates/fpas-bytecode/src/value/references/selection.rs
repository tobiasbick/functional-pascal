//! A checked writable path sharing its cell's reservation and exclusive authority.

use std::sync::Arc;

use super::{CellBorrow, ReferenceError, ReferencePathError, ReferenceStep};
use crate::Value;

/// A stored field or collection element selected from one reserved storage root.
///
/// Copies share authority and evaluated path values. Writes replace only the selected
/// value, preserving ordinary copy-on-write snapshots of every containing aggregate.
#[derive(Debug, Clone)]
pub struct SelectedReference {
    borrowed: CellBorrow,
    steps: Arc<[ReferenceStep]>,
}

impl SelectedReference {
    /// Whether two selected paths share the same reservation or active authority.
    #[must_use]
    pub fn same_authority(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.borrowed.0, &other.borrowed.0)
    }

    /// Whether two references resolve to the same root allocation, regardless of path.
    #[must_use]
    pub fn same_root(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.borrowed.0.cell, &other.borrowed.0.cell)
    }

    /// Whether the selected root is the given stable storage allocation.
    #[must_use]
    pub fn matches_cell(&self, cell: &Arc<std::sync::Mutex<Value>>) -> bool {
        Arc::ptr_eq(&self.borrowed.0.cell, cell)
    }

    /// Whether this authority belongs to the supplied VM access registry.
    #[must_use]
    pub fn uses_registry(&self, registry: &super::ReferenceRegistry) -> bool {
        self.borrowed.0.registry.same_registry(registry)
    }

    /// Select the whole root of an existing reservation or active parameter.
    #[must_use]
    pub fn root(borrowed: CellBorrow) -> Self {
        Self {
            borrowed,
            steps: Arc::from([]),
        }
    }

    /// Append an evaluated field or index after checking its type and bounds.
    ///
    /// Selection preserves the root's current authority. String indices never select writable storage.
    /// **Documentation:** `docs/pascal/language/basics/operators.md#string-indexing`
    pub fn project(&self, step: ReferenceStep) -> Result<Self, ReferencePathError> {
        let mut steps = self.steps.to_vec();
        steps.push(step);
        self.borrowed.read_selected(&steps)?;
        Ok(Self {
            borrowed: self.borrowed.clone(),
            steps: steps.into(),
        })
    }

    /// Make the selected root exclusive at synchronous callee entry.
    pub fn activate(&self) -> Result<(), ReferenceError> {
        self.borrowed.activate()
    }

    /// Read a value snapshot without reevaluating any root or index expression.
    pub fn read(&self) -> Result<Value, ReferencePathError> {
        self.borrowed.read_selected(&self.steps)
    }

    /// Replace the selected value immediately under the root's access and storage locks.
    pub fn write(&self, value: Value) -> Result<(), ReferencePathError> {
        self.borrowed.write_selected(&self.steps, value)
    }

    /// Reserve a child reference to the same path before evaluating later arguments.
    ///
    /// Parent writes are suspended while reserved; activation suspends parent reads too.
    pub fn reborrow(&self) -> Result<Self, ReferenceError> {
        Ok(Self {
            borrowed: self.borrowed.reborrow()?,
            steps: Arc::clone(&self.steps),
        })
    }

    /// Release completed authority without rolling back any selected-value changes.
    pub fn release(&self) -> Result<(), ReferenceError> {
        self.borrowed.release()
    }
}
