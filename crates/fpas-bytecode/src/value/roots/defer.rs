//! Delay synchronous maintenance until enclosing shared storage locks are gone.
use super::super::MemoryDomain;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

#[derive(Default)]
struct Deferred {
    depth: usize,
    domains: Vec<MemoryDomain>,
}
thread_local! { static DEFERRED: RefCell<Deferred> = RefCell::new(Deferred::default()); }

/// A thread-local scope, not a global interpreter or maintenance lock.
pub struct MaintenanceDeferral(PhantomData<Rc<()>>);

/// Declare shared scheduler/catalog lock ownership before taking those locks.
/// Drop this guard after all storage guards and removed holders have been released.
#[must_use]
pub fn defer_maintenance() -> MaintenanceDeferral {
    DEFERRED.with_borrow_mut(|state| state.depth += 1);
    MaintenanceDeferral(PhantomData)
}
pub(super) fn deferred(domain: &MemoryDomain) -> bool {
    DEFERRED.with_borrow_mut(|state| {
        if state.depth == 0 {
            return false;
        }
        if !state.domains.iter().any(|owner| owner.id() == domain.id()) {
            state.domains.push(domain.clone());
        }
        true
    })
}
impl Drop for MaintenanceDeferral {
    fn drop(&mut self) {
        let domains = DEFERRED.with_borrow_mut(|state| {
            state.depth -= 1;
            if state.depth == 0 {
                std::mem::take(&mut state.domains)
            } else {
                Vec::new()
            }
        });
        for domain in domains {
            domain.try_service_maintenance();
        }
    }
}
