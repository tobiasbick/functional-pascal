//! Movable logical execution roots, thread-bound access and published native waits.
use super::super::{MemoryDomain, OwnershipError, Value, release_values};
use super::{RegisteredRoots, RootKind, TraceRoots};
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

struct Published {
    running: AtomicBool,
    values: Mutex<Vec<Value>>,
}
impl TraceRoots for Published {
    fn trace_roots(&self, visit: &mut dyn FnMut(&Value)) -> Result<(), OwnershipError> {
        if self.running.load(Ordering::Acquire) {
            return Err(OwnershipError::Busy);
        }
        let values = self.values.try_lock().map_err(|e| match e {
            std::sync::TryLockError::WouldBlock => OwnershipError::Busy,
            std::sync::TryLockError::Poisoned(_) => OwnershipError::Poisoned,
        })?;
        values.as_slice().trace_roots(visit)
    }
}
impl Drop for Published {
    fn drop(&mut self) {
        release_values(std::mem::take(
            self.values.get_mut().unwrap_or_else(|e| e.into_inner()),
        ));
    }
}
/// One logical owner, moved with saved state rather than cloned with physical workers.
/// The provider never retains its registration/domain through a strong back edge.
pub struct ExecutionRoots {
    source: Arc<Published>,
    registration: RegisteredRoots,
}
impl std::fmt::Debug for ExecutionRoots {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.registration.fmt(f)
    }
}
/// Restores the enclosing native caller's unpublished execution view on return.
pub struct ExecutionResume {
    source: Arc<Published>,
    domain: MemoryDomain,
}
impl Drop for ExecutionResume {
    fn drop(&mut self) {
        let _access = self.domain.access();
        if !self.source.running.swap(true, Ordering::AcqRel) {
            let old =
                std::mem::take(&mut *self.source.values.lock().unwrap_or_else(|e| e.into_inner()));
            release_values(old);
        }
    }
}
/// An execution token. Nested same-thread callback access shares the domain gate.
pub struct ExecutionAccess {
    domain: MemoryDomain,
    outermost: bool,
    not_send: PhantomData<Rc<()>>,
}
/// Complete native roots remain published while all enclosing execution access is parked.
/// Resumption panics if interrupted committed teardown has poisoned the domain.
pub struct PublishedWait {
    domain: MemoryDomain,
    depth: usize,
    not_send: PhantomData<Rc<()>>,
}
impl MemoryDomain {
    /// Admit a logical execution owner. Retired domains permit graph access, never re-entry.
    /// # Errors
    /// Rejects execution after retirement.
    pub fn execution_roots(&self) -> Result<ExecutionRoots, OwnershipError> {
        let _access = self.access();
        {
            let mut state = self
                .0
                .roots
                .access
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            if state.retired {
                return Err(OwnershipError::Retired);
            }
            state.executions += 1;
        }
        let source = Arc::new(Published {
            running: AtomicBool::new(true),
            values: Mutex::new(Vec::new()),
        });
        let registration = self.register_roots(RootKind::Execution, source.clone());
        Ok(ExecutionRoots {
            source,
            registration,
        })
    }
    /// Release execution access only after publishing every enclosing owner and native temporary.
    /// Returns `None` when an enclosing external borrow prevents safe publication.
    pub fn publish_wait(&self) -> Option<PublishedWait> {
        let current = std::thread::current().id();
        let state = self
            .0
            .roots
            .access
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if state.owner != Some(current)
            || state.execution == 0
            || state.external != 0
            || state.collecting
        {
            return None;
        }
        drop(state);
        // Commit removals at this outer execution release even with maintenance disabled.
        {
            let access = self.access();
            drop(access);
        }
        if self.trace_roots_locked(&mut |_, _| {}).is_err() {
            return None;
        }
        let mut state = self
            .0
            .roots
            .access
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let depth = std::mem::take(&mut state.execution);
        state.parked = Some(current);
        state.owner = None;
        self.0.roots.available.notify_all();
        drop(state);
        self.try_service_maintenance();
        Some(PublishedWait {
            domain: self.clone(),
            depth,
            not_send: PhantomData,
        })
    }
}
impl ExecutionRoots {
    /// Arrange to invalidate a published caller before native result handoff ends.
    #[must_use]
    pub fn resume_marker(&self) -> ExecutionResume {
        ExecutionResume {
            source: self.source.clone(),
            domain: self.domain().clone(),
        }
    }

    /// Acquire execution exclusion before using registers, cells or native local values.
    /// # Errors
    /// Rejects retirement and collection-hook re-entry instead of waiting on itself.
    pub fn enter(&self) -> Result<ExecutionAccess, OwnershipError> {
        let domain = self.registration.domain();
        let current = std::thread::current().id();
        let mut state = domain
            .0
            .roots
            .access
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        while state.owner.is_some_and(|owner| owner != current)
            || state.parked.is_some_and(|owner| owner != current)
        {
            state = domain
                .0
                .roots
                .available
                .wait(state)
                .unwrap_or_else(|e| e.into_inner());
        }
        if state.poisoned {
            return Err(OwnershipError::Poisoned);
        }
        if state.collecting || state.parked.is_some() {
            return Err(OwnershipError::Busy);
        }
        if state.retired {
            return Err(OwnershipError::Retired);
        }
        let outermost = state.execution == 0;
        state.owner = Some(current);
        state.execution += 1;
        drop(state);
        self.invalidate();
        Ok(ExecutionAccess {
            domain: domain.clone(),
            outermost,
            not_send: PhantomData,
        })
    }
    /// Mark a view unavailable before changing live execution storage.
    pub fn invalidate(&self) {
        if self.source.running.load(Ordering::Acquire) {
            return;
        }
        let _access = self.domain().access();
        if !self.source.running.swap(true, Ordering::AcqRel) {
            let old =
                std::mem::take(&mut *self.source.values.lock().unwrap_or_else(|e| e.into_inner()));
            release_values(old);
        }
    }
    /// Publish direct roots under execution/external ownership, dropping old copies outside locks.
    pub fn publish(&self, values: Vec<Value>) {
        let _access = self.domain().access();
        let old = std::mem::replace(
            &mut *self.source.values.lock().unwrap_or_else(|e| e.into_inner()),
            values,
        );
        self.source.running.store(false, Ordering::Release);
        release_values(old);
    }
    /// Domain authority moves with the logical owner.
    #[must_use]
    pub fn domain(&self) -> &MemoryDomain {
        self.registration.domain()
    }
}
impl Drop for ExecutionRoots {
    fn drop(&mut self) {
        let mut state = self
            .registration
            .domain()
            .0
            .roots
            .access
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        state.executions -= 1;
        if state.executions == 0 && state.sessions == 0 {
            state.retired = true;
        }
    }
}
impl ExecutionAccess {
    /// Whether the caller must publish a stable view before releasing the last execution token.
    #[must_use]
    pub fn outermost(&self) -> bool {
        self.outermost
    }
    /// Service pending work only at a completed operation with every owner published.
    pub fn safe_point(&self) -> super::MaintenanceOutcome {
        self.domain.service(true)
    }
}
impl Drop for ExecutionAccess {
    fn drop(&mut self) {
        self.domain.release_access(true);
    }
}
impl Drop for PublishedWait {
    fn drop(&mut self) {
        let current = std::thread::current().id();
        let mut state = self
            .domain
            .0
            .roots
            .access
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        while state.owner.is_some_and(|owner| owner != current) {
            state = self
                .domain
                .0
                .roots
                .available
                .wait(state)
                .unwrap_or_else(|e| e.into_inner());
        }
        state.owner = Some(current);
        state.parked = None;
        state.execution += self.depth;
        // Restore token accounting so enclosing access guards can unwind safely,
        // but never return normally to execution after an interrupted commit.
        let poisoned = state.poisoned;
        drop(state);
        assert!(
            !poisoned,
            "interrupted reclamation poisoned the memory domain"
        );
    }
}
