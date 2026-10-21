//! Runtime ownership domains and weak registration of mutable value cells.
//!
//! Documentation: `docs/pascal/language/functions/closures.md`.

use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, Weak};

use super::cell::{CellStorage, OwnedCell, OwnershipError};
use super::drain::PendingValue;
use super::{Value, graph};

/// Opaque identity retained for as long as any value belongs to a domain.
///
/// Identity is independent of numeric task IDs and worker threads. Keeping the
/// token does not retain execution state, a VM, or the domain's cell registry.
#[derive(Clone)]
pub struct DomainId(Arc<()>);

impl PartialEq for DomainId {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for DomainId {}

impl Hash for DomainId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(Arc::as_ptr(&self.0), state);
    }
}

impl fmt::Debug for DomainId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("DomainId")
            .field(&Arc::as_ptr(&self.0))
            .finish()
    }
}

pub(super) struct DomainState {
    pub(super) id: DomainId,
    pub(super) cells: Mutex<HashMap<usize, Weak<CellStorage>>>,
    pub(super) roots: super::roots::DomainRoots,
    pub(super) collection: super::collector::Counters,
}

/// One logical task's cell ownership and weak registration context.
///
/// Cloning preserves the domain; creating another context creates a new identity.
/// Cells retain only an identity token and a weak link back to this context.
#[derive(Clone)]
pub struct MemoryDomain(pub(super) Arc<DomainState>);

impl Default for MemoryDomain {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for MemoryDomain {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("MemoryDomain")
            .field(self.id())
            .finish()
    }
}

impl MemoryDomain {
    /// Create a new identity unrelated to any other live or retired task.
    #[must_use]
    pub fn new() -> Self {
        Self(Arc::new(DomainState {
            id: DomainId(Arc::new(())),
            cells: Mutex::new(HashMap::new()),
            roots: super::roots::DomainRoots::default(),
            collection: super::collector::Counters::default(),
        }))
    }

    /// Borrow the identity checked by every cell read and write.
    #[must_use]
    pub fn id(&self) -> &DomainId {
        &self.0.id
    }

    /// Validate and allocate a cell without retaining its contents in a registry.
    ///
    /// # Errors
    /// Rejects foreign local values, escaping references and unavailable cells.
    pub fn new_cell(&self, value: Value) -> Result<OwnedCell, OwnershipError> {
        let value = PendingValue::new(value);
        let _access = self.access();
        graph::validate_storable(self, &value)?;
        let cell = OwnedCell::new(self, value.into_inner());
        self.0
            .cells
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(cell.identity(), Arc::downgrade(&cell.0));
        super::collector::increment(&self.0.collection.allocated, 1);
        self.note_cell_allocation();
        Ok(cell)
    }

    /// Snapshot live registered cells, pruning expired entries.
    ///
    /// These temporary owners are administrative handles, not execution roots.
    #[must_use]
    pub fn live_cells(&self) -> Vec<OwnedCell> {
        let mut registry = self
            .0
            .cells
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut live = Vec::with_capacity(registry.len());
        registry.retain(|_, cell| {
            if let Some(cell) = cell.upgrade() {
                live.push(OwnedCell(cell));
                true
            } else {
                false
            }
        });
        live
    }

    /// Count live weak entries after pruning expired registrations.
    #[must_use]
    pub fn registered_count(&self) -> usize {
        let mut registry = self
            .0
            .cells
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        registry.retain(|_, cell| cell.strong_count() != 0);
        registry.len()
    }
}
