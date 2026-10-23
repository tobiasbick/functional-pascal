//! Encapsulated mutable cells with checked domain access and atomic replacement.
//!
//! Documentation: `docs/pascal/language/functions/closures.md`.

use std::fmt;
use std::ops::Deref;
use std::sync::{Arc, Mutex, MutexGuard, TryLockError, Weak};

use super::domain::{DomainId, DomainState, MemoryDomain};
use super::drain::PendingValue;
use super::{Value, graph};

/// Ownership validation or a cell access could not be completed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OwnershipError {
    /// A reachable local value belongs to another domain.
    ForeignDomain,
    /// Complete marking or its commit scratch buffers could not be prepared.
    ScratchUnavailable,
    /// A temporary variable reference would escape into retained storage.
    EscapingReference,
    /// A nonblocking cell snapshot could not obtain access.
    Busy,
    /// An interrupted access left the cell unavailable.
    Poisoned,
    /// Execution cannot re-enter a closed domain.
    Retired,
    /// Another replacement won after an update took its immutable snapshot.
    ConcurrentMutation,
}

impl fmt::Display for OwnershipError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ScratchUnavailable => {
                "collection scratch storage is unavailable or its bound was exceeded"
            }
            Self::ForeignDomain => "local value belongs to another memory domain",
            Self::EscapingReference => "variable reference cannot escape into a cell",
            Self::Busy => "mutable cell is busy",
            Self::Poisoned => "mutable cell is unavailable after an interrupted access",
            Self::Retired => "memory domain is retired",
            Self::ConcurrentMutation => "mutable cell changed during the update",
        })
    }
}

impl std::error::Error for OwnershipError {}

/// Distinguishes a caller's failed transformation from an ownership/access error.
#[derive(Debug)]
pub enum CellUpdateError<E> {
    /// Validation or commit failed without replacing the current contents.
    Ownership(OwnershipError),
    /// The transformation rejected its immutable input snapshot.
    Update(E),
}

impl<E: fmt::Display> fmt::Display for CellUpdateError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ownership(error) => error.fmt(formatter),
            Self::Update(error) => error.fmt(formatter),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for CellUpdateError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(match self {
            Self::Ownership(error) => error,
            Self::Update(error) => error,
        })
    }
}

pub(super) struct CellStorage {
    owner: DomainId,
    domain: Weak<DomainState>,
    // Snapshotting an Arc never recursively clones deeply nested wrapper values.
    value: Mutex<Arc<Value>>,
}

impl Drop for CellStorage {
    fn drop(&mut self) {
        if let Some(domain) = self.domain.upgrade() {
            super::collector::increment(&domain.collection.released, 1);
            domain
                .cells
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .remove(&(std::ptr::from_ref(self) as usize));
        }
        let slot = self
            .value
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let old = std::mem::replace(slot, Arc::new(Value::Unit));
        if let Some(value) = Arc::into_inner(old) {
            super::drain::values(&mut vec![value]);
        }
    }
}

/// Shared cell whose contents can only be read or replaced by its domain.
#[derive(Clone)]
pub struct OwnedCell(pub(super) Arc<CellStorage>);

/// A weak lifetime observation that never keeps a cell or its domain alive.
#[derive(Debug, Clone)]
pub struct WeakOwnedCell(Weak<CellStorage>);

impl WeakOwnedCell {
    /// Acquire a temporary owner if the cell has not been released.
    #[must_use]
    pub fn upgrade(&self) -> Option<OwnedCell> {
        self.0.upgrade().map(OwnedCell)
    }
}

/// Immutable payload snapshot acquired through a domain-checked cell read.
///
/// Retaining a snapshot preserves its value independently of later replacement.
/// Final release uses the same iterative teardown as mutable cell storage.
#[derive(Debug, Clone)]
pub struct CellSnapshot(Option<Arc<Value>>);

impl CellSnapshot {
    fn arc(&self) -> &Arc<Value> {
        let Some(value) = self.0.as_ref() else {
            unreachable!("live cell snapshot has a payload");
        };
        value
    }

    pub(super) fn from_arc(value: Arc<Value>) -> Self {
        Self(Some(value))
    }

    fn into_arc(mut self) -> Arc<Value> {
        let Some(value) = self.0.take() else {
            unreachable!("a live cell snapshot has a payload");
        };
        value
    }
}

impl Deref for CellSnapshot {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        self.arc()
    }
}

impl Drop for CellSnapshot {
    fn drop(&mut self) {
        if let Some(snapshot) = self.0.take()
            && let Some(value) = Arc::into_inner(snapshot)
            && !is_inline_scalar(&value)
        {
            super::drain::values(&mut vec![value]);
        }
    }
}

impl fmt::Debug for OwnedCell {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OwnedCell")
            .field("identity", &self.identity())
            .field("owner", self.owner())
            .finish_non_exhaustive()
    }
}

impl OwnedCell {
    pub(super) fn new(domain: &MemoryDomain, value: Value) -> Self {
        Self(Arc::new(CellStorage {
            owner: domain.id().clone(),
            domain: Arc::downgrade(&domain.0),
            value: Mutex::new(Arc::new(value)),
        }))
    }

    /// Borrow identity metadata without granting access to the payload.
    #[must_use]
    pub fn owner(&self) -> &DomainId {
        &self.0.owner
    }

    /// Stable allocation identity while this handle remains alive.
    #[must_use]
    pub fn identity(&self) -> usize {
        Arc::as_ptr(&self.0) as usize
    }

    /// Test whether two handles share the same mutable storage.
    #[must_use]
    pub fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    /// Observe the cell's lifetime without retaining its payload.
    #[must_use]
    pub fn downgrade(&self) -> WeakOwnedCell {
        WeakOwnedCell(Arc::downgrade(&self.0))
    }

    fn check_owner(&self, domain: &MemoryDomain) -> Result<(), OwnershipError> {
        if self.owner() == domain.id() {
            Ok(())
        } else {
            Err(OwnershipError::ForeignDomain)
        }
    }

    pub(super) fn lock(
        &self,
        nonblocking: bool,
    ) -> Result<MutexGuard<'_, Arc<Value>>, OwnershipError> {
        if nonblocking {
            self.0.value.try_lock().map_err(|error| match error {
                TryLockError::WouldBlock => OwnershipError::Busy,
                TryLockError::Poisoned(_) => OwnershipError::Poisoned,
            })
        } else {
            self.0.value.lock().map_err(|_| OwnershipError::Poisoned)
        }
    }

    pub(super) fn graph_snapshot(&self) -> Result<CellSnapshot, OwnershipError> {
        Ok(CellSnapshot(Some(Arc::clone(&*self.lock(true)?))))
    }

    /// Retain an immutable snapshot after checking the caller's domain.
    ///
    /// # Errors
    /// Rejects foreign ownership or a poisoned cell.
    pub fn read_snapshot(&self, domain: &MemoryDomain) -> Result<CellSnapshot, OwnershipError> {
        self.check_owner(domain)?;
        let _access = domain.operation_access();
        Ok(CellSnapshot(Some(Arc::clone(&*self.lock(false)?))))
    }

    /// Retain a snapshot without waiting on busy storage.
    ///
    /// # Errors
    /// Rejects foreign ownership, a busy cell, or a poisoned cell.
    pub fn try_read_snapshot(&self, domain: &MemoryDomain) -> Result<CellSnapshot, OwnershipError> {
        self.check_owner(domain)?;
        let _access = domain.try_access()?;
        self.graph_snapshot()
    }

    /// Copy the payload after checking the caller's domain.
    ///
    /// # Errors
    /// Rejects foreign ownership or a poisoned cell.
    pub fn read(&self, domain: &MemoryDomain) -> Result<Value, OwnershipError> {
        let mut value = Value::Unit;
        if let Some(integer) = self.read_integer_or_value(domain, &mut value)? {
            return Ok(Value::Integer(integer));
        }
        Ok(value)
    }

    /// Read once, returning an integer directly or storing another payload in `value`.
    ///
    /// Integer reads and failed reads leave `value` unchanged. Other reads replace it only
    /// after releasing cell storage and domain access, so dropping its old contents is safe.
    ///
    /// # Errors
    /// Rejects foreign ownership or a poisoned cell.
    pub fn read_integer_or_value(
        &self,
        domain: &MemoryDomain,
        value: &mut Value,
    ) -> Result<Option<i64>, OwnershipError> {
        self.check_owner(domain)?;
        let snapshot = {
            let access = domain.operation_access();
            let slot = self.lock(false)?;
            if let Value::Integer(integer) = &**slot {
                return Ok(Some(*integer));
            }
            if is_inline_scalar(&slot) {
                let scalar = (**slot).clone();
                drop(slot);
                drop(access);
                *value = scalar;
                return Ok(None);
            }
            // Shared payloads must still be cloned after releasing storage and domain access.
            CellSnapshot(Some(Arc::clone(&slot)))
        };
        *value = (*snapshot).clone();
        Ok(None)
    }

    /// Copy the payload without waiting on busy storage.
    ///
    /// # Errors
    /// Rejects foreign ownership, a busy cell, or a poisoned cell.
    pub fn try_read(&self, domain: &MemoryDomain) -> Result<Value, OwnershipError> {
        let snapshot = self.try_read_snapshot(domain)?;
        Ok((*snapshot).clone())
    }

    /// Validate a new graph before atomically replacing this cell's contents.
    ///
    /// Removed contents are released after dropping the cell lock.
    ///
    /// # Errors
    /// Rejects foreign ownership, escaping references, or unavailable storage.
    pub fn replace(&self, domain: &MemoryDomain, value: Value) -> Result<(), OwnershipError> {
        self.replace_inner(domain, value, false)
    }

    /// Replace a validated payload without waiting for the commit lock.
    ///
    /// # Errors
    /// Reports the same validation failures as `replace`, plus busy storage.
    pub fn try_replace(&self, domain: &MemoryDomain, value: Value) -> Result<(), OwnershipError> {
        self.replace_inner(domain, value, true)
    }

    fn replace_inner(
        &self,
        domain: &MemoryDomain,
        value: Value,
        nonblocking: bool,
    ) -> Result<(), OwnershipError> {
        let value = PendingValue::new(value);
        self.check_owner(domain)?;
        let _access = if nonblocking {
            Some(domain.try_access()?)
        } else {
            domain.operation_access()
        };
        if !is_inline_scalar(&value) {
            graph::validate_storable(domain, &value)?;
        }
        let mut slot = self.lock(nonblocking)?;
        match Arc::get_mut(&mut slot) {
            // No snapshot shares the payload, so it can change in place.
            Some(current) => {
                let old = std::mem::replace(current, value.into_inner());
                drop(slot);
                if !is_inline_scalar(&old) {
                    super::drain::release_value(old);
                }
            }
            None => {
                let old = std::mem::replace(&mut *slot, Arc::new(value.into_inner()));
                drop(slot);
                drop(CellSnapshot(Some(old)));
            }
        }
        domain.note_cell_mutation();
        Ok(())
    }

    /// Transform an immutable snapshot, validate it, and commit only if unchanged.
    ///
    /// The transformation runs outside cell locks. A racing replacement causes an
    /// explicit error instead of overwriting it with a stale update.
    ///
    /// # Errors
    /// Distinguishes transformation failure from validation or concurrent mutation.
    pub fn update<E>(
        &self,
        domain: &MemoryDomain,
        update: impl FnOnce(&Value) -> Result<Value, E>,
    ) -> Result<(), CellUpdateError<E>> {
        self.update_inner(domain, update, false)
    }

    /// Transform and commit without waiting for a busy cell.
    ///
    /// # Errors
    /// Reports the same failures as `update`, plus busy storage.
    pub fn try_update<E>(
        &self,
        domain: &MemoryDomain,
        update: impl FnOnce(&Value) -> Result<Value, E>,
    ) -> Result<(), CellUpdateError<E>> {
        self.update_inner(domain, update, true)
    }

    fn update_inner<E>(
        &self,
        domain: &MemoryDomain,
        update: impl FnOnce(&Value) -> Result<Value, E>,
        nonblocking: bool,
    ) -> Result<(), CellUpdateError<E>> {
        self.check_owner(domain)
            .map_err(CellUpdateError::Ownership)?;
        let _access = if nonblocking {
            domain.try_access().map_err(CellUpdateError::Ownership)?
        } else {
            domain.access()
        };
        let snapshot = CellSnapshot(Some(Arc::clone(
            &*self.lock(nonblocking).map_err(CellUpdateError::Ownership)?,
        )));
        let next = PendingValue::new(update(&snapshot).map_err(CellUpdateError::Update)?);
        graph::validate_storable(domain, &next).map_err(CellUpdateError::Ownership)?;
        let next = CellSnapshot(Some(Arc::new(next.into_inner())));
        let old = {
            let mut slot = self.lock(nonblocking).map_err(CellUpdateError::Ownership)?;
            if !Arc::ptr_eq(&slot, snapshot.arc()) {
                return Err(CellUpdateError::Ownership(
                    OwnershipError::ConcurrentMutation,
                ));
            }
            std::mem::replace(&mut *slot, next.into_arc())
        };
        drop(CellSnapshot(Some(old)));
        domain.note_cell_mutation();
        Ok(())
    }

    pub(super) fn drain_unique(self, pending: &mut Vec<Value>) {
        if let Some(mut cell) = Arc::into_inner(self.0) {
            let slot = cell
                .value
                .get_mut()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let old = std::mem::replace(slot, Arc::new(Value::Unit));
            if let Some(value) = Arc::into_inner(old) {
                pending.push(value);
            }
        }
    }
}

/// Inline scalars own no further storage, graph edges or iterative teardown.
fn is_inline_scalar(value: &Value) -> bool {
    matches!(
        value,
        Value::Integer(_) | Value::Real(_) | Value::Boolean(_) | Value::Unit | Value::OptionNone
    )
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod rejected_tests;
