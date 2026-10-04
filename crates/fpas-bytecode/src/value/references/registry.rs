//! Shared root reservations and atomic checks around cell access.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use super::borrow::BorrowToken;
use super::{CellBorrow, ReferenceError, ReferencePathError, ReferenceStep, path};
use crate::Value;

/// Access registry shared by all workers and callbacks of one VM.
///
/// Root identity is the cell allocation, so different names and aggregate paths
/// cannot bypass a reservation. Value snapshots continue to use ordinary cloning.
#[derive(Clone, Default)]
pub struct ReferenceRegistry {
    state: Arc<Mutex<RegistryState>>,
}

#[derive(Default)]
struct RegistryState {
    next_identity: u64,
    roots: HashMap<usize, Vec<Lease>>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Reserved,
    Active,
}

struct Lease {
    identity: u64,
    parent: Option<u64>,
    phase: Phase,
}

impl ReferenceRegistry {
    /// Whether both handles enforce access through the same registry state.
    pub(super) fn same_registry(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.state, &other.state)
    }
    /// Reserve a root before evaluating the remaining call arguments.
    pub fn reserve(&self, cell: &Arc<Mutex<Value>>) -> Result<CellBorrow, ReferenceError> {
        self.reserve_from(cell, None)
    }

    /// Reserve a fresh root or a child of the currently active authority.
    pub(super) fn reserve_from(
        &self,
        cell: &Arc<Mutex<Value>>,
        parent: Option<&CellBorrow>,
    ) -> Result<CellBorrow, ReferenceError> {
        let root = root_identity(cell);
        let mut state = self.lock();
        if let Some(parent) = parent
            && !state.roots.get(&root).is_some_and(|leases| {
                leases
                    .iter()
                    .any(|lease| lease.identity == parent.0.identity)
            })
        {
            return Err(ReferenceError::Released);
        }
        match (
            state.roots.get(&root).and_then(|leases| leases.last()),
            parent,
        ) {
            (None, None) => {}
            (Some(lease), None) => return Err(phase_error(lease.phase)),
            (None, Some(_)) => return Err(ReferenceError::Released),
            (Some(lease), Some(parent)) => {
                if lease.identity != parent.0.identity {
                    return Err(ReferenceError::Suspended);
                }
                if lease.phase != Phase::Active {
                    return Err(ReferenceError::NotActive);
                }
            }
        }
        let identity = state
            .next_identity
            .checked_add(1)
            .ok_or(ReferenceError::IdentityLimit)?;
        state.next_identity = identity;
        state.roots.entry(root).or_default().push(Lease {
            identity,
            parent: parent.map(|parent| parent.0.identity),
            phase: Phase::Reserved,
        });
        Ok(CellBorrow(Arc::new(BorrowToken {
            registry: self.clone(),
            cell: Arc::clone(cell),
            identity,
            _parent: parent.cloned(),
        })))
    }

    /// Read a value snapshot through an ordinary cell alias.
    pub fn read(&self, cell: &Arc<Mutex<Value>>) -> Result<Value, ReferenceError> {
        self.read_with(cell, None)
    }

    /// Replace a root through an ordinary cell alias.
    pub fn write(&self, cell: &Arc<Mutex<Value>>, value: Value) -> Result<(), ReferenceError> {
        self.write_with(cell, None, value)
    }

    /// Copy a root while holding the access check and storage lock together.
    pub(super) fn read_with(
        &self,
        cell: &Arc<Mutex<Value>>,
        authority: Option<u64>,
    ) -> Result<Value, ReferenceError> {
        let state = self.lock();
        state.check_access(root_identity(cell), authority, false)?;
        let value = cell
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
        Ok(value)
    }

    /// Replace a root after atomically checking the supplied authority.
    pub(super) fn write_with(
        &self,
        cell: &Arc<Mutex<Value>>,
        authority: Option<u64>,
        value: Value,
    ) -> Result<(), ReferenceError> {
        let state = self.lock();
        state.check_access(root_identity(cell), authority, true)?;
        let mut stored = cell.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let previous = std::mem::replace(&mut *stored, value);
        // Destructors may release other references. Drop values outside both locks.
        drop(stored);
        drop(state);
        drop(previous);
        Ok(())
    }

    /// Read the checked path without copying its containing aggregates.
    pub(super) fn read_selected(
        &self,
        cell: &Arc<Mutex<Value>>,
        identity: u64,
        steps: &[ReferenceStep],
    ) -> Result<Value, ReferencePathError> {
        let state = self.lock();
        state.check_access(root_identity(cell), Some(identity), false)?;
        let stored = cell.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        path::read(&stored, steps).cloned()
    }

    /// Commit a selected update atomically; invalid paths leave stored values unchanged.
    pub(super) fn write_selected(
        &self,
        cell: &Arc<Mutex<Value>>,
        identity: u64,
        steps: &[ReferenceStep],
        mut value: Value,
    ) -> Result<(), ReferencePathError> {
        let state = self.lock();
        state.check_access(root_identity(cell), Some(identity), true)?;
        let mut stored = cell.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut updated = stored.clone();
        let result = path::replace(&mut updated, steps, &mut value);
        if result.is_ok() {
            std::mem::swap(&mut *stored, &mut updated);
        }
        drop(stored);
        drop(state);
        // Drop replaced values after releasing the access and storage locks.
        drop(updated);
        drop(value);
        result
    }

    /// Make the current reservation exclusive at callee entry.
    pub(super) fn activate(
        &self,
        cell: &Arc<Mutex<Value>>,
        identity: u64,
    ) -> Result<(), ReferenceError> {
        let mut state = self.lock();
        let leases = state
            .roots
            .get_mut(&root_identity(cell))
            .ok_or(ReferenceError::Released)?;
        if !leases.iter().any(|lease| lease.identity == identity) {
            return Err(ReferenceError::Released);
        }
        let lease = leases.last_mut().ok_or(ReferenceError::Released)?;
        if lease.identity != identity {
            return Err(ReferenceError::Suspended);
        }
        if lease.phase == Phase::Active {
            return Err(ReferenceError::AlreadyActive);
        }
        lease.phase = Phase::Active;
        Ok(())
    }

    /// Remove one authority without changing its stored value.
    pub(super) fn release(
        &self,
        cell: &Arc<Mutex<Value>>,
        identity: u64,
    ) -> Result<(), ReferenceError> {
        let root = root_identity(cell);
        let mut state = self.lock();
        let Some(leases) = state.roots.get_mut(&root) else {
            return Ok(());
        };
        let Some(position) = leases.iter().position(|lease| lease.identity == identity) else {
            return Ok(());
        };
        if position + 1 != leases.len() {
            return Err(ReferenceError::Suspended);
        }
        leases.pop();
        if leases.is_empty() {
            state.roots.remove(&root);
        }
        Ok(())
    }

    fn lock(&self) -> MutexGuard<'_, RegistryState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl RegistryState {
    fn check_access(
        &self,
        root: usize,
        authority: Option<u64>,
        write: bool,
    ) -> Result<(), ReferenceError> {
        let Some(leases) = self.roots.get(&root) else {
            return if authority.is_some() {
                Err(ReferenceError::Released)
            } else {
                Ok(())
            };
        };
        if let Some(identity) = authority
            && !leases.iter().any(|lease| lease.identity == identity)
        {
            return Err(ReferenceError::Released);
        }
        let Some(lease) = leases.last() else {
            return Err(ReferenceError::Released);
        };
        match lease.phase {
            Phase::Reserved if write => Err(ReferenceError::Reserved),
            Phase::Reserved if authority == Some(lease.identity) || authority == lease.parent => {
                Ok(())
            }
            Phase::Active if authority == Some(lease.identity) => Ok(()),
            _ if authority.is_some() => Err(ReferenceError::Suspended),
            _ => Err(ReferenceError::Exclusive),
        }
    }
}

fn root_identity(cell: &Arc<Mutex<Value>>) -> usize {
    Arc::as_ptr(cell) as usize
}

fn phase_error(phase: Phase) -> ReferenceError {
    match phase {
        Phase::Reserved => ReferenceError::Reserved,
        Phase::Active => ReferenceError::Exclusive,
    }
}

#[cfg(test)]
mod identity_tests {
    use super::*;

    #[test]
    fn identity_limit_never_wraps_or_leaves_a_root_reserved() {
        let registry = ReferenceRegistry::default();
        registry.lock().next_identity = u64::MAX - 1;
        let root = Arc::new(Mutex::new(Value::Integer(1)));
        let final_borrow = registry.reserve(&root).unwrap();
        final_borrow.activate().unwrap();
        final_borrow.release().unwrap();
        assert_eq!(
            registry.reserve(&root).unwrap_err(),
            ReferenceError::IdentityLimit
        );
        registry.write(&root, Value::Integer(2)).unwrap();
        assert_eq!(registry.read(&root).unwrap(), Value::Integer(2));
        assert_eq!(final_borrow.read(), Err(ReferenceError::Released));
    }
}
