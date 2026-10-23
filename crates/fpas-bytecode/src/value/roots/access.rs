//! The single per-domain gate. Never hold its mutex while tracing or dropping values.

use super::super::MemoryDomain;
use super::registration::RootEntry;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::thread::ThreadId;

#[derive(Default)]
pub(super) struct AccessState {
    pub owner: Option<ThreadId>,
    pub external: usize,
    pub execution: usize,
    pub parked: Option<ThreadId>,
    pub collecting: bool,
    pub poisoned: bool,
    pub removing: usize,
    pub pending: Vec<Arc<RootEntry>>,
    pub dirty: bool,
    pub executions: usize,
    pub sessions: usize,
    pub retired: bool,
}

/// Reentrant external access, excluding execution and maintenance on other threads.
///
/// Registered providers and cell locks must be released before this guard. Collection
/// hooks use their prepared root view, never re-enter graph access or FPAS execution.
pub struct DomainAccess {
    domain: MemoryDomain,
    not_send: PhantomData<Rc<()>>,
}

impl MemoryDomain {
    /// Enter guarded external access. Re-entry from a collection hook is forbidden.
    #[must_use]
    pub fn access(&self) -> DomainAccess {
        // Count every borrow: it may outlive execution or span a safe point/native wait.
        let current = super::current_thread();
        let mut state = self
            .0
            .roots
            .access
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        while state.owner.is_some_and(|owner| owner != current) {
            state = self
                .0
                .roots
                .available
                .wait(state)
                .unwrap_or_else(|e| e.into_inner());
        }
        assert!(
            !state.poisoned,
            "interrupted reclamation poisoned the memory domain"
        );
        assert!(
            !state.collecting,
            "collection hooks cannot re-enter external domain access"
        );
        state.owner = Some(current);
        state.external += 1;
        DomainAccess {
            domain: self.clone(),
            not_send: PhantomData,
        }
    }

    /// Try external access without waiting under a diagnostic/storage deadline.
    /// # Errors
    /// Returns Busy when execution/maintenance or another accessor owns this domain.
    pub fn try_access(&self) -> Result<DomainAccess, super::super::OwnershipError> {
        use super::super::OwnershipError;
        let current = super::current_thread();
        let mut state = self
            .0
            .roots
            .access
            .try_lock()
            .map_err(|_| OwnershipError::Busy)?;
        if state.poisoned {
            return Err(OwnershipError::Poisoned);
        }
        if state.collecting || state.owner.is_some_and(|owner| owner != current) {
            return Err(OwnershipError::Busy);
        }
        state.owner = Some(current);
        state.external += 1;
        Ok(DomainAccess {
            domain: self.clone(),
            not_send: PhantomData,
        })
    }

    /// Access for one bounded cell operation, reusing the executing owner's exclusion.
    ///
    /// Only the executing owner thread observes its own token, and only it can park or
    /// reach a safe point. Callers must therefore keep the guard inside one operation that
    /// never escapes, parks, reaches a safe point or runs FPAS/host code. Every other
    /// caller gets a counted borrow.
    pub(in crate::value) fn operation_access(&self) -> Option<DomainAccess> {
        if self.0.roots.exec_owner.load(Ordering::Acquire) == super::thread_token() {
            None
        } else {
            Some(self.access())
        }
    }

    pub(in crate::value) fn require_collection(&self) -> Result<(), super::super::OwnershipError> {
        use super::super::OwnershipError;
        let state = self
            .0
            .roots
            .access
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if state.poisoned {
            return Err(OwnershipError::Poisoned);
        }
        if !state.collecting
            || state.owner != Some(super::current_thread())
            || state.external != 0
            || state.removing != 0
        {
            return Err(OwnershipError::Busy);
        }
        Ok(())
    }
    pub(in crate::value) fn poison_collection(&self) {
        let mut state = self
            .0
            .roots
            .access
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        state.poisoned = true;
        self.refresh_fast_owner(&state);
    }

    /// Publish the executing owner for pressure accounting. Requires the state lock.
    ///
    /// Holds only while this thread owns an unparked execution gate with no collection or
    /// poison, so only its mutations can skip the quiescent-state lookup.
    pub(super) fn refresh_fast_owner(&self, state: &AccessState) {
        let token = if state.execution != 0
            && !state.collecting
            && !state.poisoned
            && state.parked.is_none()
            && state.owner == Some(super::current_thread())
        {
            super::thread_token()
        } else {
            0
        };
        self.0.roots.exec_owner.store(token, Ordering::Release);
    }

    pub(super) fn release_root(&self, root: Arc<RootEntry>) {
        let current = super::current_thread();
        let mut state = self
            .0
            .roots
            .access
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if state.owner.is_some_and(|owner| owner != current) && !state.collecting {
            // Root retirement may commit during execution. A short in-flight counter
            // excludes collection while values are released, without blocking execution
            // or waiting on an opaque native call. No registry/cell lock is held here.
            state.removing += 1;
            drop(state);
            drop(root);
            let mut state = self
                .0
                .roots
                .access
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            state.removing -= 1;
            drop(state);
            self.try_service_maintenance();
        } else if state.owner.is_some_and(|owner| owner != current) {
            state.pending.push(root);
            drop(state);
            self.request_maintenance();
        } else if state.external != 0 || state.collecting {
            state.pending.push(root);
        } else {
            state.owner = Some(current);
            state.external = 1;
            drop(state);
            drop(root);
            drop(DomainAccess {
                domain: self.clone(),
                not_send: PhantomData,
            });
        }
    }

    pub(super) fn release_access(&self, execution: bool) {
        let mut state = self
            .0
            .roots
            .access
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if execution {
            state.execution -= 1;
        } else {
            state.external -= 1;
        }
        self.refresh_fast_owner(&state);
        if state.external != 0 || state.collecting {
            return;
        }
        // Nothing to retire and an enclosing execution still owns the domain.
        if state.execution != 0 && state.pending.is_empty() {
            return;
        }
        // Keep ownership while destructors run outside every protocol/storage mutex.
        while !state.pending.is_empty() {
            state.external = 1;
            let pending = std::mem::take(&mut state.pending);
            drop(state);
            drop(pending);
            state = self
                .0
                .roots
                .access
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            state.external = 0;
        }
        if state.execution != 0 {
            return;
        }
        // Request recording takes this same mutex. A request after this release sees
        // a quiescent domain and services it; earlier requests are handed off below.
        state.owner = None;
        self.0.roots.available.notify_all();
        drop(state);
        self.try_service_maintenance();
    }
}
impl Drop for DomainAccess {
    fn drop(&mut self) {
        self.domain.release_access(false);
    }
}
