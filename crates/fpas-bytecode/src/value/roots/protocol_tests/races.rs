use super::*;
use std::sync::Barrier;

#[test]
fn requests_during_collection_get_a_subsequent_pass_and_do_not_block_unrelated_domains() {
    struct Paused {
        entered: Barrier,
        leave: Barrier,
        calls: AtomicUsize,
    }
    impl DomainMaintenance for Paused {
        fn maintain(&self, _: &MemoryDomain, _: &[Value]) -> Result<(), OwnershipError> {
            if self.calls.fetch_add(1, Ordering::SeqCst) == 0 {
                self.entered.wait();
                self.leave.wait();
            }
            Ok(())
        }
    }
    let domain = MemoryDomain::new();
    let probe = Arc::new(Paused {
        entered: Barrier::new(2),
        leave: Barrier::new(2),
        calls: AtomicUsize::new(0),
    });
    domain.set_maintenance(probe.clone(), MaintenancePolicy::Automatic);
    let other = domain.clone();
    let collector = std::thread::spawn(move || other.request_maintenance());
    probe.entered.wait();
    domain.request_maintenance();
    let unrelated = MemoryDomain::new();
    let owner = unrelated.execution_roots().unwrap();
    let access = owner.enter().unwrap();
    owner.publish(vec![Value::Integer(3)]);
    drop(access);
    probe.leave.wait();
    collector.join().unwrap();
    assert_eq!(probe.calls.load(Ordering::SeqCst), 2);
    assert_eq!(
        domain.serviced_generation(),
        domain.maintenance_generation()
    );
}
#[test]
fn root_removal_by_another_thread_does_not_wait_for_running_execution() {
    let domain = MemoryDomain::new();
    let probe = automatic(&domain);
    let a = domain.root_values(RootKind::Host, vec![Value::Integer(1)]);
    let owner = domain.execution_roots().unwrap();
    let access = owner.enter().unwrap();
    std::thread::spawn(move || drop(a)).join().unwrap();
    assert_eq!(probe.passes.load(Ordering::SeqCst), 0);
    owner.publish(vec![Value::Integer(2)]);
    access.safe_point();
    assert!(
        probe
            .observed
            .lock()
            .unwrap()
            .last()
            .unwrap()
            .iter()
            .all(|value| *value != Value::Integer(1))
    );
    drop(access);
}
#[test]
fn generation_saturation_does_not_hide_a_request_made_inside_a_pass() {
    struct Again(AtomicUsize);
    impl DomainMaintenance for Again {
        fn maintain(&self, domain: &MemoryDomain, _: &[Value]) -> Result<(), OwnershipError> {
            if self.0.fetch_add(1, Ordering::SeqCst) == 0 {
                domain.request_maintenance();
            }
            Ok(())
        }
    }
    let domain = MemoryDomain::new();
    let probe = Arc::new(Again(AtomicUsize::new(0)));
    domain.set_maintenance(probe.clone(), MaintenancePolicy::Automatic);
    domain.0.roots.generation.store(u64::MAX, Ordering::SeqCst);
    domain.request_maintenance();
    assert_eq!(probe.0.load(Ordering::SeqCst), 2);
    assert_eq!(domain.serviced_generation(), u64::MAX);
    assert!(!domain.maintenance_pending());
}
#[test]
fn panic_unwind_releases_exclusion_and_keeps_the_request_pending() {
    struct Panic;
    impl DomainMaintenance for Panic {
        fn maintain(&self, _: &MemoryDomain, _: &[Value]) -> Result<(), OwnershipError> {
            panic!("probe panic")
        }
    }
    let domain = MemoryDomain::new();
    domain.set_maintenance(Arc::new(Panic), MaintenancePolicy::Explicit);
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(
            || domain.request_explicit_maintenance()
        ))
        .is_err()
    );
    assert!(domain.maintenance_pending());
    let probe = automatic(&domain);
    assert_eq!(probe.passes.load(Ordering::SeqCst), 1);
    assert!(!domain.maintenance_pending());
}

#[test]
fn root_release_and_external_access_wait_for_exclusive_collection() {
    use std::sync::mpsc;
    struct Paused {
        entered: Barrier,
        leave: Barrier,
        snapshots: Mutex<Vec<Vec<Value>>>,
    }
    impl DomainMaintenance for Paused {
        fn maintain(&self, _: &MemoryDomain, roots: &[Value]) -> Result<(), OwnershipError> {
            let first = {
                let mut snapshots = self.snapshots.lock().unwrap();
                snapshots.push(roots.to_vec());
                snapshots.len() == 1
            };
            if first {
                self.entered.wait();
                self.leave.wait();
            }
            Ok(())
        }
    }
    let domain = MemoryDomain::new();
    let a = domain.root_values(RootKind::Host, vec![Value::Integer(1)]);
    let _b = domain.root_values(RootKind::Host, vec![Value::Integer(2)]);
    let hook = Arc::new(Paused {
        entered: Barrier::new(2),
        leave: Barrier::new(2),
        snapshots: Mutex::new(Vec::new()),
    });
    domain.set_maintenance(hook.clone(), MaintenancePolicy::Explicit);
    std::thread::scope(|scope| {
        let collect = scope.spawn(|| domain.request_explicit_maintenance());
        hook.entered.wait();
        assert!(matches!(domain.try_access(), Err(OwnershipError::Busy)));
        std::thread::spawn(move || drop(a)).join().unwrap();
        let (started_tx, started_rx) = mpsc::channel();
        let (access_tx, access_rx) = mpsc::channel();
        let access_domain = &domain;
        let accessor = scope.spawn(move || {
            started_tx.send(()).unwrap();
            let _access = access_domain.access();
            access_tx.send(()).unwrap();
        });
        started_rx.recv().unwrap();
        assert!(access_rx.try_recv().is_err());
        hook.leave.wait();
        assert!(matches!(
            collect.join().unwrap(),
            MaintenanceOutcome::Serviced(_)
        ));
        accessor.join().unwrap();
    });
    let snapshots = hook.snapshots.lock().unwrap();
    assert_eq!(snapshots.len(), 2);
    assert!(snapshots[0].contains(&Value::Integer(1)));
    assert_eq!(snapshots[1], vec![Value::Integer(2)]);
    assert_eq!(
        domain.serviced_generation(),
        domain.maintenance_generation()
    );
}
