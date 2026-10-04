//! Reference authority retained independently from register-window copies.

use std::sync::{Arc, Mutex};

use super::{ReferenceError, ReferencePathError, ReferenceRegistry, ReferenceStep};
use crate::Value;

/// Shared authority for a reserved or active synchronous cell reference.
///
/// Clones carry the same authority, rather than creating a second root borrow.
/// Call completion releases authority explicitly even if a register retains a
/// clone; dropping the last clone also releases an abandoned reservation.
#[derive(Clone)]
pub struct CellBorrow(pub(super) Arc<BorrowToken>);

/// Authority identity and the retained storage/parent allocations.
pub(super) struct BorrowToken {
    pub(super) registry: ReferenceRegistry,
    pub(super) cell: Arc<Mutex<Value>>,
    pub(super) identity: u64,
    // A child keeps its parent alive until it has restored the parent's authority.
    pub(super) _parent: Option<CellBorrow>,
}

impl CellBorrow {
    /// Enter the callee and exclude all unrelated reads and writes of the root.
    pub fn activate(&self) -> Result<(), ReferenceError> {
        self.0.registry.activate(&self.0.cell, self.0.identity)
    }

    /// Reserve a forwarded reference, suspending parent writes during argument evaluation.
    pub fn reborrow(&self) -> Result<Self, ReferenceError> {
        self.0.registry.reserve_from(&self.0.cell, Some(self))
    }

    /// Read a value snapshot using this reference's authority.
    pub fn read(&self) -> Result<Value, ReferenceError> {
        self.0
            .registry
            .read_with(&self.0.cell, Some(self.0.identity))
    }

    /// Replace the root immediately; no copy-back or rollback occurs on release.
    pub fn write(&self, value: Value) -> Result<(), ReferenceError> {
        self.0
            .registry
            .write_with(&self.0.cell, Some(self.0.identity), value)
    }

    /// Read a selected path using this root's authority.
    pub(super) fn read_selected(
        &self,
        steps: &[ReferenceStep],
    ) -> Result<Value, ReferencePathError> {
        self.0
            .registry
            .read_selected(&self.0.cell, self.0.identity, steps)
    }

    /// Replace a selected value without separating its access check from the update.
    pub(super) fn write_selected(
        &self,
        steps: &[ReferenceStep],
        value: Value,
    ) -> Result<(), ReferencePathError> {
        self.0
            .registry
            .write_selected(&self.0.cell, self.0.identity, steps, value)
    }

    /// Release a completed call's authority, including authority in retained clones.
    ///
    /// A parent with a live child cannot be released; release children first.
    /// Releasing the same authority more than once has no effect.
    pub fn release(&self) -> Result<(), ReferenceError> {
        self.0.registry.release(&self.0.cell, self.0.identity)
    }
}

impl Drop for BorrowToken {
    fn drop(&mut self) {
        let _ = self.registry.release(&self.cell, self.identity);
        // Unwind uniquely held parent tokens iteratively, including deep call chains.
        let mut parent = self._parent.take();
        while let Some(borrowed) = parent {
            match Arc::try_unwrap(borrowed.0) {
                Ok(mut token) => {
                    let _ = token.registry.release(&token.cell, token.identity);
                    parent = token._parent.take();
                }
                Err(_) => break,
            }
        }
    }
}

impl std::fmt::Debug for CellBorrow {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("CellBorrow")
    }
}
