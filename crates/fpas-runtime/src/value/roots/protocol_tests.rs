//! Deterministic ownership interleavings without enabling cell reclamation.
mod lifecycle;
mod races;

use super::*;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

#[derive(Default)]
struct Probe {
    passes: AtomicUsize,
    observed: Mutex<Vec<Vec<Value>>>,
}
impl DomainMaintenance for Probe {
    fn maintain(&self, domain: &MemoryDomain, roots: &[Value]) -> Result<(), OwnershipError> {
        assert_eq!(domain.phase(), DomainPhase::Collecting);
        self.passes.fetch_add(1, Ordering::SeqCst);
        self.observed.lock().unwrap().push(roots.to_vec());
        Ok(())
    }
}
fn automatic(domain: &MemoryDomain) -> Arc<Probe> {
    let probe = Arc::new(Probe::default());
    domain.set_maintenance(probe.clone(), MaintenancePolicy::Automatic);
    probe
}
#[test]
fn execution_defers_requests_until_a_complete_safe_point() {
    let domain = MemoryDomain::new();
    let probe = automatic(&domain);
    let owner = domain.execution_roots().unwrap();
    let access = owner.enter().unwrap();
    domain.request_maintenance();
    assert_eq!(probe.passes.load(Ordering::SeqCst), 0);
    assert_eq!(
        access.safe_point(),
        MaintenanceOutcome::Busy(OwnershipError::Busy)
    );
    owner.publish(vec![Value::Integer(7)]);
    assert!(matches!(
        access.safe_point(),
        MaintenanceOutcome::Serviced(_)
    ));
    assert_eq!(
        probe.observed.lock().unwrap().last().unwrap(),
        &vec![Value::Integer(7)]
    );
    drop(access);
}
#[test]
fn nested_external_release_services_once_after_removals_commit() {
    let domain = MemoryDomain::new();
    let probe = automatic(&domain);
    let a = domain.root_values(RootKind::Host, vec![Value::Integer(1)]);
    let b = domain.root_values(RootKind::Host, vec![Value::Integer(2)]);
    let outer = domain.access();
    let inner = domain.access();
    drop(a);
    drop(inner);
    assert_eq!(domain.root_count(), 2);
    assert_eq!(probe.passes.load(Ordering::SeqCst), 0);
    drop(outer);
    assert_eq!(domain.root_count(), 1);
    assert_eq!(probe.observed.lock().unwrap()[0], vec![Value::Integer(2)]);
    assert_eq!(probe.passes.load(Ordering::SeqCst), 1);
    drop(b);
}
#[test]
fn explicit_policy_keeps_ordinary_notifications_pending_until_requested() {
    let domain = MemoryDomain::new();
    let probe = Arc::new(Probe::default());
    domain.set_maintenance(probe.clone(), MaintenancePolicy::Explicit);
    domain.request_maintenance();
    assert_eq!(probe.passes.load(Ordering::SeqCst), 0);
    assert!(matches!(
        domain.request_explicit_maintenance(),
        MaintenanceOutcome::Serviced(_)
    ));
    assert_eq!(probe.passes.load(Ordering::SeqCst), 1);
}
#[test]
fn shared_storage_scope_defers_service_until_after_its_mutex_is_released() {
    struct Locked(Arc<Mutex<()>>, AtomicUsize);
    impl DomainMaintenance for Locked {
        fn maintain(&self, _: &MemoryDomain, _: &[Value]) -> Result<(), OwnershipError> {
            assert!(
                self.0.try_lock().is_ok(),
                "shared storage lock leaked into maintenance"
            );
            self.1.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }
    let domain = MemoryDomain::new();
    let storage = Arc::new(Mutex::new(()));
    let probe = Arc::new(Locked(storage.clone(), AtomicUsize::new(0)));
    domain.set_maintenance(probe.clone(), MaintenancePolicy::Automatic);
    let scope = defer_maintenance();
    let lock = storage.lock().unwrap();
    domain.request_maintenance();
    assert_eq!(probe.1.load(Ordering::SeqCst), 0);
    drop(lock);
    drop(scope);
    assert_eq!(probe.1.load(Ordering::SeqCst), 1);
}
#[test]
fn published_native_wait_services_while_waiter_stays_blocked() {
    let domain = MemoryDomain::new();
    let probe = automatic(&domain);
    let owner = domain.execution_roots().unwrap();
    let (published, ready) = std::sync::mpsc::channel();
    let (resume, resumed) = std::sync::mpsc::channel();
    let other = domain.clone();
    let waiter = std::thread::spawn(move || {
        let access = owner.enter().unwrap();
        owner.publish(vec![Value::Integer(9)]);
        let wait = other.publish_wait().expect("complete native publication");
        published.send(()).unwrap();
        resumed.recv().unwrap();
        drop(wait);
        assert_eq!(other.phase(), DomainPhase::Executing);
        owner.publish(vec![]);
        drop(access);
    });
    ready.recv().unwrap();
    domain.request_maintenance();
    assert_eq!(probe.passes.load(Ordering::SeqCst), 1);
    assert_eq!(probe.observed.lock().unwrap()[0], vec![Value::Integer(9)]);
    resume.send(()).unwrap();
    waiter.join().unwrap();
}
#[test]
fn execution_owner_reenters_access_while_other_threads_stay_excluded() {
    let domain = MemoryDomain::new();
    let owner = domain.execution_roots().unwrap();
    let access = owner.enter().unwrap();
    // The executing thread re-enters without waiting; its release leaves ownership intact.
    drop(domain.access());
    let cell = domain.new_cell(Value::Integer(1)).unwrap();
    cell.replace(&domain, Value::Integer(2)).unwrap();
    assert_eq!(cell.read(&domain).unwrap(), Value::Integer(2));
    let other = domain.clone();
    std::thread::spawn(move || {
        assert_eq!(other.try_access().err(), Some(OwnershipError::Busy));
    })
    .join()
    .unwrap();
    drop(access);
    let other = domain.clone();
    std::thread::spawn(move || assert!(other.try_access().is_ok()))
        .join()
        .unwrap();
}
#[test]
fn collection_clears_the_execution_owner_fast_path_and_restores_it_afterwards() {
    struct Observe(std::sync::atomic::AtomicU64);
    impl DomainMaintenance for Observe {
        fn maintain(&self, domain: &MemoryDomain, _: &[Value]) -> Result<(), OwnershipError> {
            self.0.store(
                domain.0.roots.exec_owner.load(Ordering::SeqCst),
                Ordering::SeqCst,
            );
            Ok(())
        }
    }
    let domain = MemoryDomain::new();
    let observed = Arc::new(Observe(std::sync::atomic::AtomicU64::new(u64::MAX)));
    domain.set_maintenance(observed.clone(), MaintenancePolicy::Automatic);
    let owner = domain.execution_roots().unwrap();
    let access = owner.enter().unwrap();
    assert_eq!(
        domain.0.roots.exec_owner.load(Ordering::SeqCst),
        thread_token()
    );
    owner.publish(vec![]);
    domain.request_maintenance();
    assert!(matches!(
        access.safe_point(),
        MaintenanceOutcome::Serviced(_)
    ));
    assert_eq!(observed.0.load(Ordering::SeqCst), 0);
    assert_eq!(
        domain.0.roots.exec_owner.load(Ordering::SeqCst),
        thread_token()
    );
    drop(access);
    assert_eq!(domain.0.roots.exec_owner.load(Ordering::SeqCst), 0);
}
#[test]
fn opaque_execution_and_incomplete_providers_never_authorize_maintenance() {
    let domain = MemoryDomain::new();
    let probe = automatic(&domain);
    let owner = domain.execution_roots().unwrap();
    let access = owner.enter().unwrap();
    assert!(domain.publish_wait().is_none());
    domain.request_maintenance();
    drop(access);
    assert_eq!(probe.passes.load(Ordering::SeqCst), 0);
    owner.publish(vec![Value::Integer(1)]);
    assert_eq!(probe.passes.load(Ordering::SeqCst), 1);
}
