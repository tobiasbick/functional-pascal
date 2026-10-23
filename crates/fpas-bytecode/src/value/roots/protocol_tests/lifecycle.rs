use super::*;

#[test]
fn execution_nested_external_access_defers_safe_point_collection() {
    let domain = MemoryDomain::new();
    let probe = automatic(&domain);
    let owner = domain.execution_roots().unwrap();
    let execution = owner.enter().unwrap();
    let external = domain.access();
    owner.publish(vec![]);
    domain.request_maintenance();
    assert_eq!(execution.safe_point(), MaintenanceOutcome::Deferred);
    assert_eq!(probe.passes.load(Ordering::SeqCst), 0);
    drop(external);
    assert!(matches!(
        execution.safe_point(),
        MaintenanceOutcome::Serviced(_)
    ));
}

#[test]
fn execution_nested_external_access_prevents_published_wait() {
    let domain = MemoryDomain::new();
    let owner = domain.execution_roots().unwrap();
    let _execution = owner.enter().unwrap();
    let external = domain.access();
    owner.publish(vec![]);
    assert!(domain.publish_wait().is_none());
    drop(external);
    assert!(domain.publish_wait().is_some());
}

#[test]
fn execution_nested_external_access_outlives_execution_exclusion() {
    let domain = MemoryDomain::new();
    let probe = automatic(&domain);
    let owner = domain.execution_roots().unwrap();
    let execution = owner.enter().unwrap();
    let external = domain.access();
    owner.publish(vec![]);
    domain.request_maintenance();
    drop(execution);
    assert_eq!(probe.passes.load(Ordering::SeqCst), 0);
    let other = domain.clone();
    std::thread::spawn(move || {
        assert_eq!(other.try_access().err(), Some(OwnershipError::Busy));
    })
    .join()
    .unwrap();
    drop(external);
    assert_eq!(probe.passes.load(Ordering::SeqCst), 1);
    let other = domain.clone();
    std::thread::spawn(move || assert!(other.try_access().is_ok()))
        .join()
        .unwrap();
}

#[test]
fn retired_domain_services_nonfinal_root_release_without_a_worker_or_vm() {
    let domain = MemoryDomain::new();
    let probe = automatic(&domain);
    let owner = domain.execution_roots().unwrap();
    let a = domain.root_values(RootKind::Host, vec![Value::Integer(1)]);
    let b = domain.root_values(RootKind::Host, vec![Value::Integer(2)]);
    drop(owner);
    assert_eq!(domain.phase(), DomainPhase::Retired);
    let before = probe.passes.load(Ordering::SeqCst);
    drop(domain);
    drop(a);
    assert!(probe.passes.load(Ordering::SeqCst) > before);
    assert_eq!(b.with_values(|values| values[0].clone()), Value::Integer(2));
    assert!(b.domain().execution_roots().is_err());
    assert_eq!(
        b.domain().serviced_generation(),
        b.domain().maintenance_generation()
    );
}
#[test]
fn session_activity_survives_invocation_return_but_closes_on_shutdown() {
    let (domain, session) = MemoryDomain::new_session();
    let owner = domain.execution_roots().unwrap();
    owner.publish(vec![]);
    drop(owner);
    assert_eq!(domain.phase(), DomainPhase::Idle);
    let second = domain.execution_roots().unwrap();
    second.publish(vec![]);
    drop(second);
    drop(session);
    assert_eq!(domain.phase(), DomainPhase::Retired);
    assert!(matches!(
        domain.execution_roots(),
        Err(OwnershipError::Retired)
    ));
    let cell = domain.new_cell(Value::Integer(7)).unwrap();
    let _access = domain.access();
    cell.replace(&domain, Value::Integer(8)).unwrap();
    assert_eq!(cell.read(&domain).unwrap(), Value::Integer(8));
}
#[test]
fn migration_moves_the_owner_and_saved_roots_without_thread_affinity() {
    let domain = MemoryDomain::new();
    let probe = automatic(&domain);
    let owner = domain.execution_roots().unwrap();
    let access = owner.enter().unwrap();
    owner.publish(vec![Value::Integer(42)]);
    drop(access);
    domain.request_maintenance();
    assert_eq!(probe.observed.lock().unwrap()[0], vec![Value::Integer(42)]);
    std::thread::spawn(move || {
        let access = owner.enter().unwrap();
        owner.publish(vec![Value::Integer(43)]);
        drop(access);
    })
    .join()
    .unwrap();
    assert_eq!(domain.phase(), DomainPhase::Retired);
}
#[test]
fn collection_hook_reentry_is_rejected_without_waiting_on_its_own_gate() {
    struct Reentry(std::sync::Weak<ExecutionRoots>);
    impl DomainMaintenance for Reentry {
        fn maintain(&self, _: &MemoryDomain, _: &[Value]) -> Result<(), OwnershipError> {
            assert!(matches!(
                self.0.upgrade().unwrap().enter(),
                Err(OwnershipError::Busy)
            ));
            Ok(())
        }
    }
    let domain = MemoryDomain::new();
    let owner = Arc::new(domain.execution_roots().unwrap());
    owner.publish(vec![]);
    // Hooks retain no domain or registration through a strong administrative edge.
    domain.set_maintenance(
        Arc::new(Reentry(Arc::downgrade(&owner))),
        MaintenancePolicy::Explicit,
    );
    assert!(matches!(
        domain.request_explicit_maintenance(),
        MaintenanceOutcome::Serviced(_)
    ));
    domain.set_maintenance(Arc::new(Probe::default()), MaintenancePolicy::Explicit);
}
