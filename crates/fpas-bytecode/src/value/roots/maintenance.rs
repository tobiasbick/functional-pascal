//! Generation handshake and exclusive prepared-root maintenance.

use super::super::{MemoryDomain, OwnershipError, Value};
use std::sync::{Arc, atomic::Ordering};

/// Trusted domain-local maintenance. Implementations must not retain their domain,
/// enter FPAS, acquire shared runtime locks, or change registrations from this hook.
/// Removed values must be released outside cell locks. Reclamation uses the shared
/// maintenance-only domain interface and this complete prepared root view.
pub trait DomainMaintenance: Send + Sync {
    /// Run under exclusive collection ownership, using a complete prepared root view.
    /// # Errors
    /// Incomplete traversal must return an error before changing any graph edge.
    fn maintain(&self, domain: &MemoryDomain, roots: &[Value]) -> Result<(), OwnershipError>;
}
/// Production activation remains off; AP48.6 uses explicit requests first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaintenancePolicy {
    /// Run only requests explicitly made through the maintenance entry point.
    Explicit,
    /// Service root-release/mutation requests as well; production activation is AP48.7.
    Automatic,
}
/// Whether a request was completed or remains pending for a later safe point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaintenanceOutcome {
    /// No eligible request/handler is configured.
    Disabled,
    /// An active owner will service the pending request later.
    Deferred,
    /// A provider/traversal was incomplete; no request was acknowledged.
    Busy(OwnershipError),
    /// All work through this generation completed under exclusive ownership.
    Serviced(u64),
}

struct CollectionGuard(MemoryDomain, bool);
impl Drop for CollectionGuard {
    fn drop(&mut self) {
        if !self.1 {
            return;
        }
        let mut state = self
            .0
            .0
            .roots
            .access
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        state.collecting = false;
        if state.external == 0 && state.execution == 0 {
            state.owner = None;
            self.0.0.roots.available.notify_all();
        }
    }
}
impl MemoryDomain {
    /// Install trusted maintenance context without a strong domain back-reference.
    /// VM domains install explicit-only reclamation; automatic collection stays off.
    pub fn set_maintenance(&self, hook: Arc<dyn DomainMaintenance>, policy: MaintenancePolicy) {
        let _access = self.access();
        let old = self
            .0
            .roots
            .maintenance
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .replace(hook);
        self.0
            .roots
            .automatic
            .store(policy == MaintenancePolicy::Automatic, Ordering::Release);
        self.0.roots.installed.store(true, Ordering::Release);
        drop(old);
    }
    /// Explicitly request the same route used by release/mutation and safe points.
    pub fn request_explicit_maintenance(&self) -> MaintenanceOutcome {
        self.record_request(true);
        self.try_service_maintenance()
    }
    pub(super) fn record_request(&self, explicit: bool) {
        let mut state = self
            .0
            .roots
            .access
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _ = self
            .0
            .roots
            .generation
            .try_update(Ordering::AcqRel, Ordering::Acquire, |old| {
                Some(old.saturating_add(1))
            });
        state.dirty = true;
        self.0.roots.pending.store(true, Ordering::Release);
        if explicit {
            self.0.roots.explicit.store(true, Ordering::Release);
        }
        drop(state);
        if !explicit {
            self.try_service_maintenance();
        }
    }
    /// Cheap batch-boundary check; disabled production maintenance takes no mutex.
    #[must_use]
    pub fn maintenance_pending(&self) -> bool {
        self.0.roots.pending.load(Ordering::Acquire)
            && self.0.roots.installed.load(Ordering::Acquire)
            && (self.0.roots.automatic.load(Ordering::Acquire)
                || self.0.roots.explicit.load(Ordering::Acquire))
    }
    /// Last completely serviced request generation. Busy passes do not acknowledge work.
    #[must_use]
    pub fn serviced_generation(&self) -> u64 {
        self.0.roots.serviced.load(Ordering::Acquire)
    }
    /// Try synchronous service on a quiescent domain. Never waits for an active owner.
    pub fn try_service_maintenance(&self) -> MaintenanceOutcome {
        self.service(false)
    }
    pub(super) fn service(&self, safe_point: bool) -> MaintenanceOutcome {
        if !self.maintenance_pending() {
            return MaintenanceOutcome::Disabled;
        }
        if super::defer::deferred(self) {
            return MaintenanceOutcome::Deferred;
        }
        let current = std::thread::current().id();
        let mut state = self
            .0
            .roots
            .access
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if state.poisoned {
            return MaintenanceOutcome::Busy(OwnershipError::Poisoned);
        }
        if state.collecting
            || state.removing != 0
            || state.external != 0
            || state.owner.is_some_and(|owner| owner != current)
            || (state.execution != 0 && !safe_point)
        {
            return MaintenanceOutcome::Deferred;
        }
        state.owner = Some(current);
        state.collecting = true;
        drop(state);
        let mut collection = CollectionGuard(self.clone(), true);
        loop {
            // Deferred removals stay roots until committed here; destructors hold no locks.
            loop {
                let pending = {
                    let mut state = self
                        .0
                        .roots
                        .access
                        .lock()
                        .unwrap_or_else(|e| e.into_inner());
                    std::mem::take(&mut state.pending)
                };
                if pending.is_empty() {
                    break;
                }
                drop(pending);
            }
            let generation = {
                let mut state = self
                    .0
                    .roots
                    .access
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                state.dirty = false;
                self.maintenance_generation()
            };
            let mut values = Vec::new();
            let mut scratch = Ok(());
            let result = self
                .trace_roots_locked(&mut |_, value| {
                    if scratch.is_ok() {
                        if values.try_reserve(1).is_ok() {
                            values.push(value.clone());
                        } else {
                            scratch = Err(OwnershipError::ScratchUnavailable);
                        }
                    }
                })
                .and(scratch);
            let hook = self
                .0
                .roots
                .maintenance
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clone();
            let result = result.and_then(|()| match hook {
                Some(hook) => hook.maintain(self, &values),
                None => Err(OwnershipError::Busy),
            });
            super::super::release_values(values);
            let mut state = self
                .0
                .roots
                .access
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            if let Err(error) = result {
                state.dirty = true;
                return MaintenanceOutcome::Busy(error);
            }
            self.0.roots.serviced.store(generation, Ordering::Release);
            if state.dirty || !state.pending.is_empty() {
                drop(state);
                continue;
            }
            // This mutex also guards request recording and the owner-release handshake.
            self.0.roots.pending.store(false, Ordering::Release);
            self.0.roots.explicit.store(false, Ordering::Release);
            state.collecting = false;
            if state.external == 0 && state.execution == 0 {
                state.owner = None;
                self.0.roots.available.notify_all();
            }
            collection.1 = false;
            return MaintenanceOutcome::Serviced(generation);
        }
    }
}
