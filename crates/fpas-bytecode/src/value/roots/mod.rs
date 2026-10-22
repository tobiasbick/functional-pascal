//! Registered external roots and the shared execution-root enumeration contract.
//!
//! Registries are weak. A root retains its domain, while registry entries and deferred
//! removal payloads never retain the domain. Prepared views feed a separate graph
//! walker and collector; root providers never implement their own reclamation.

mod access;
mod defer;
mod execution;
mod lifecycle;
mod maintenance;
#[cfg(test)]
mod protocol_tests;
mod registration;
mod scheduling;
#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, Weak};

use super::{MemoryDomain, OwnershipError, Value};

thread_local! {
    static THREAD: std::thread::ThreadId = std::thread::current().id();
}

static NEXT_TOKEN: AtomicU64 = AtomicU64::new(1);

thread_local! {
    static TOKEN: u64 = NEXT_TOKEN.fetch_add(1, Ordering::Relaxed);
}

/// Nonzero numeric identity of the calling thread; `u64::MAX` never matches an owner.
fn thread_token() -> u64 {
    TOKEN.try_with(|token| *token).unwrap_or(u64::MAX)
}

/// Identity of the calling thread without cloning its handle on every domain access.
fn current_thread() -> std::thread::ThreadId {
    THREAD
        .try_with(|id| *id)
        .unwrap_or_else(|_| std::thread::current().id())
}

pub use access::DomainAccess;
pub use defer::{MaintenanceDeferral, defer_maintenance};
pub use execution::{ExecutionAccess, ExecutionResume, ExecutionRoots, PublishedWait};
pub use lifecycle::{DomainPhase, DomainSession};
pub use maintenance::{DomainMaintenance, MaintenanceOutcome, MaintenancePolicy};
pub use registration::{RegisteredRoots, RootValues};
pub use scheduling::CollectionSchedule;

/// The holder responsible for retaining a graph outside ordinary register execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootKind {
    /// Stable execution roots published by a saved or stopped owner.
    Execution,
    /// Public embedding export.
    Host,
    /// Live debugger or failure inspection.
    Inspection,
    /// A detached debugger evaluation and its construction temporaries.
    Evaluation,
    /// Native callers, inputs and intermediate callback results.
    Native,
    /// Idle session globals.
    Globals,
    /// An unused or claimed selection case.
    Selection,
}

/// Complete direct Value edges held by one execution or external owner.
///
/// The caller supplies stable ownership before enumeration. Implementations must
/// report unavailable storage instead of silently omitting roots. Nested value edges
/// are followed by the shared graph walker, not by individual holders.
pub trait TraceRoots {
    /// Visit every retained direct value without invoking FPAS code.
    ///
    /// # Errors
    /// Reports busy or poisoned storage when a stable view cannot be obtained.
    fn trace_roots(&self, visit: &mut dyn FnMut(&Value)) -> Result<(), OwnershipError>;
}

impl TraceRoots for [Value] {
    fn trace_roots(&self, visit: &mut dyn FnMut(&Value)) -> Result<(), OwnershipError> {
        for value in self {
            visit(value);
        }
        Ok(())
    }
}

#[derive(Default)]
pub(super) struct DomainRoots {
    entries: Mutex<HashMap<usize, Weak<registration::RootEntry>>>,
    access: Mutex<access::AccessState>,
    available: Condvar,
    generation: AtomicU64,
    pending: AtomicBool,
    explicit: AtomicBool,
    automatic: AtomicBool,
    scheduled: AtomicBool,
    /// Token of the thread that holds an unparked execution gate, else zero.
    ///
    /// Written only under the access mutex; lets that thread re-enter graph access without it.
    exec_owner: AtomicU64,
    installed: AtomicBool,
    serviced: AtomicU64,
    maintenance: Mutex<Option<Arc<dyn DomainMaintenance>>>,
}

impl MemoryDomain {
    /// Register an owned set of direct roots, retaining this domain until release.
    #[must_use]
    pub fn root_values(&self, kind: RootKind, values: Vec<Value>) -> RootValues {
        RootValues::new(self.clone(), kind, values)
    }

    /// Register an independently owned, synchronized root provider.
    ///
    /// The provider must not retain this registration or its domain through a
    /// strong administrative back edge. Execution owners use `TraceRoots` directly.
    #[must_use]
    pub fn register_roots(
        &self,
        kind: RootKind,
        source: Arc<dyn TraceRoots + Send + Sync>,
    ) -> RegisteredRoots {
        RegisteredRoots::new(self.clone(), kind, source)
    }

    /// Enumerate registered holders under guarded domain access.
    ///
    /// # Errors
    /// Reports unavailable provider storage; a partial view is never a tracing proof.
    pub fn trace_registered_roots(
        &self,
        visit: &mut dyn FnMut(RootKind, &Value),
    ) -> Result<(), OwnershipError> {
        let _access = self.access();
        self.trace_roots_locked(visit)
    }

    pub(super) fn trace_roots_locked(
        &self,
        visit: &mut dyn FnMut(RootKind, &Value),
    ) -> Result<(), OwnershipError> {
        let entries = {
            let mut roots = self
                .0
                .roots
                .entries
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            let mut entries = Vec::new();
            entries
                .try_reserve(roots.len())
                .map_err(|_| OwnershipError::ScratchUnavailable)?;
            roots.retain(|_, root| {
                if let Some(root) = root.upgrade() {
                    entries.push(root);
                    true
                } else {
                    false
                }
            });
            entries
        };
        for entry in entries {
            entry
                .source
                .trace_roots(&mut |value| visit(entry.kind, value))?;
        }
        Ok(())
    }

    /// Number of live registered holders, including deferred removals.
    #[must_use]
    pub fn root_count(&self) -> usize {
        let mut roots = self
            .0
            .roots
            .entries
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        roots.retain(|_, root| root.strong_count() != 0);
        roots.len()
    }

    /// Request maintenance after an external edge has been disconnected.
    ///
    /// Generation exhaustion is sticky: it never wraps to an older generation.
    /// The maintenance policy determines whether this also services collection.
    pub fn request_maintenance(&self) {
        self.record_request(false);
    }

    /// Observe removal/mutation requests independently of the original VM lifetime.
    #[must_use]
    pub fn maintenance_generation(&self) -> u64 {
        self.0.roots.generation.load(Ordering::Acquire)
    }
}
