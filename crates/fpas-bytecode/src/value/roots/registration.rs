//! Weakly registered root owners and mutable native/evaluation root sets.

use std::fmt;
use std::sync::{Arc, Mutex, Weak};

use super::super::{MemoryDomain, OwnershipError, Value, domain::DomainState, release_values};
use super::{RootKind, TraceRoots};

pub(super) struct RootEntry {
    domain: Weak<DomainState>,
    pub(super) kind: RootKind,
    pub(super) source: Arc<dyn TraceRoots + Send + Sync>,
}

impl Drop for RootEntry {
    fn drop(&mut self) {
        if let Some(domain) = self.domain.upgrade() {
            let mut entries = domain
                .roots
                .entries
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            entries.remove(&(std::ptr::from_ref(self) as usize));
            drop(entries);
            MemoryDomain(domain).request_maintenance();
        }
    }
}

/// A registered holder retaining its domain without retaining an execution context.
pub struct RegisteredRoots {
    domain: MemoryDomain,
    entry: Option<Arc<RootEntry>>,
}

impl fmt::Debug for RegisteredRoots {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RegisteredRoots")
            .field("domain", &self.domain)
            .finish_non_exhaustive()
    }
}

impl Clone for RegisteredRoots {
    #[expect(
        clippy::expect_used,
        reason = "entry is present throughout a live registration; only Drop takes it"
    )]
    fn clone(&self) -> Self {
        let _access = self.domain.access();
        let entry = self.entry.as_ref().expect("live registration");
        Self::new(self.domain.clone(), entry.kind, Arc::clone(&entry.source))
    }
}

impl RegisteredRoots {
    pub(super) fn new(
        domain: MemoryDomain,
        kind: RootKind,
        source: Arc<dyn TraceRoots + Send + Sync>,
    ) -> Self {
        let _access = domain.access();
        let entry = Arc::new(RootEntry {
            domain: Arc::downgrade(&domain.0),
            kind,
            source,
        });
        domain
            .0
            .roots
            .entries
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(Arc::as_ptr(&entry) as usize, Arc::downgrade(&entry));
        Self {
            domain: domain.clone(),
            entry: Some(entry),
        }
    }

    /// Retain domain authority independently of session lifetime.
    #[must_use]
    pub fn domain(&self) -> &MemoryDomain {
        &self.domain
    }
}

impl Drop for RegisteredRoots {
    fn drop(&mut self) {
        if let Some(entry) = self.entry.take() {
            self.domain.release_root(entry);
        }
    }
}

struct Values(Mutex<Vec<Value>>);

impl TraceRoots for Values {
    fn trace_roots(&self, visit: &mut dyn FnMut(&Value)) -> Result<(), OwnershipError> {
        let values = self.0.try_lock().map_err(|e| match e {
            std::sync::TryLockError::WouldBlock => OwnershipError::Busy,
            std::sync::TryLockError::Poisoned(_) => OwnershipError::Poisoned,
        })?;
        values.as_slice().trace_roots(visit)
    }
}

impl Drop for Values {
    fn drop(&mut self) {
        release_values(std::mem::take(
            self.0.get_mut().unwrap_or_else(|e| e.into_inner()),
        ));
    }
}

/// An explicitly registered set for host, debugger and native temporary values.
#[derive(Clone)]
pub struct RootValues {
    registration: RegisteredRoots,
    values: Arc<Values>,
}

impl fmt::Debug for RootValues {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.registration.fmt(f)
    }
}

impl RootValues {
    pub(super) fn new(domain: MemoryDomain, kind: RootKind, values: Vec<Value>) -> Self {
        let values = Arc::new(Values(Mutex::new(values)));
        let registration = domain.register_roots(kind, values.clone());
        Self {
            registration,
            values,
        }
    }

    /// Domain authority for guarded internal graph operations.
    #[must_use]
    pub fn domain(&self) -> &MemoryDomain {
        self.registration.domain()
    }

    /// Borrow the registered values while external domain access is held.
    pub fn with_values<R>(&self, read: impl FnOnce(&[Value]) -> R) -> R {
        let _access = self.domain().access();
        let values = self.values.0.lock().unwrap_or_else(|e| e.into_inner());
        read(&values)
    }

    /// Publish a replacement set before dropping disconnected values outside locks.
    pub fn replace(&self, values: Vec<Value>) {
        let _access = self.domain().access();
        let old = {
            let mut previous = self.values.0.lock().unwrap_or_else(|e| e.into_inner());
            std::mem::replace(&mut *previous, values)
        };
        self.domain().request_maintenance();
        release_values(old);
    }

    /// Retain another construction temporary without disconnecting existing edges.
    pub fn push(&self, value: Value) {
        let _access = self.domain().access();
        self.values
            .0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(value);
    }
}
