//! Actual collection under independent pressure triggers and mandatory boundaries.
use super::*;
use crate::{
    CollectionLimits, DomainMaintenance, MaintenanceOutcome, MaintenancePolicy, OwnedCell,
    OwnershipError, RootKind, Value,
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

struct Collector;
impl DomainMaintenance for Collector {
    fn maintain(&self, domain: &MemoryDomain, roots: &[Value]) -> Result<(), OwnershipError> {
        domain
            .reclaim_unreachable(roots, CollectionLimits::default())
            .map(|_| ())
    }
}
fn schedule(allocations: usize, mutations: usize, batches: usize) -> CollectionSchedule {
    CollectionSchedule {
        allocations: NonZeroUsize::new(allocations).unwrap(),
        mutations: NonZeroUsize::new(mutations).unwrap(),
        safe_points: NonZeroUsize::new(batches).unwrap(),
    }
}
fn domain(config: CollectionSchedule) -> MemoryDomain {
    let domain = MemoryDomain::new();
    domain.set_maintenance(Arc::new(Collector), MaintenancePolicy::Scheduled(config));
    domain
}
fn cycle(domain: &MemoryDomain) -> OwnedCell {
    let cell = domain.new_cell(Value::Unit).unwrap();
    cell.replace(domain, Value::option_some(Value::Cell(cell.clone())))
        .unwrap();
    cell
}
#[test]
fn allocation_pressure_collects_only_after_complete_publication_and_resets() {
    let domain = domain(schedule(2, usize::MAX, usize::MAX));
    let owner = domain.execution_roots().unwrap();
    let access = owner.enter().unwrap();
    drop(cycle(&domain));
    owner.publish(vec![]);
    assert_eq!(access.batch_safe_point(), MaintenanceOutcome::Disabled);
    drop(cycle(&domain));
    assert_eq!(domain.collection_stats().cleared_cells, 0);
    owner.publish(vec![]);
    assert!(matches!(
        access.batch_safe_point(),
        MaintenanceOutcome::Serviced(_)
    ));
    assert_eq!(domain.collection_stats().live_cells, 0);
    drop(cycle(&domain));
    owner.publish(vec![]);
    assert_eq!(access.batch_safe_point(), MaintenanceOutcome::Disabled);
    drop(cycle(&domain));
    owner.publish(vec![]);
    assert!(matches!(
        access.batch_safe_point(),
        MaintenanceOutcome::Serviced(_)
    ));
    assert_eq!(domain.collection_stats().cleared_cells, 4);
}
#[test]
fn successful_mutations_have_an_independent_running_threshold() {
    let domain = domain(schedule(usize::MAX, 2, usize::MAX));
    let owner = domain.execution_roots().unwrap();
    let access = owner.enter().unwrap();
    let cell = cycle(&domain);
    let weak = cell.downgrade();
    owner.publish(vec![]);
    assert_eq!(access.batch_safe_point(), MaintenanceOutcome::Disabled);
    cell.replace(&domain, Value::option_some(Value::Cell(cell.clone())))
        .unwrap();
    drop(cell);
    owner.publish(vec![]);
    assert!(matches!(
        access.batch_safe_point(),
        MaintenanceOutcome::Serviced(_)
    ));
    assert!(weak.upgrade().is_none());
}
#[test]
fn periodic_batches_reclaim_dropped_values_without_new_allocations_or_mutations() {
    let domain = domain(schedule(usize::MAX, usize::MAX, 2));
    let owner = domain.execution_roots().unwrap();
    let access = owner.enter().unwrap();
    let cell = cycle(&domain);
    let weak = cell.downgrade();
    owner.publish(vec![Value::Cell(cell)]);
    assert_eq!(access.batch_safe_point(), MaintenanceOutcome::Disabled);
    assert!(matches!(
        access.batch_safe_point(),
        MaintenanceOutcome::Serviced(_)
    ));
    owner.publish(vec![]);
    for _ in 0..8 {
        assert_eq!(access.safe_point(), MaintenanceOutcome::Disabled);
    }
    assert_eq!(access.batch_safe_point(), MaintenanceOutcome::Disabled);
    assert!(weak.upgrade().is_some());
    assert!(matches!(
        access.batch_safe_point(),
        MaintenanceOutcome::Serviced(_)
    ));
    assert!(weak.upgrade().is_none());
}
#[test]
fn retired_nonfinal_root_release_and_external_mutation_bypass_all_thresholds() {
    let domain = domain(schedule(usize::MAX, usize::MAX, usize::MAX));
    let owner = domain.execution_roots().unwrap();
    let access = domain.access();
    let a = cycle(&domain);
    let b = cycle(&domain);
    let c = cycle(&domain);
    b.replace(
        &domain,
        Value::Array(vec![Value::Cell(b.clone()), Value::Cell(c.clone())].into()),
    )
    .unwrap();
    let weak_a = a.downgrade();
    let weak_b = b.downgrade();
    let weak_c = c.downgrade();
    let a = domain.root_values(RootKind::Host, vec![Value::Cell(a)]);
    let root_b = domain.root_values(RootKind::Host, vec![Value::Cell(b.clone())]);
    drop(c);
    drop(owner);
    drop(access);
    assert_eq!(domain.phase(), crate::DomainPhase::Retired);
    drop(a);
    assert!(weak_a.upgrade().is_none());
    assert!(weak_b.upgrade().is_some() && weak_c.upgrade().is_some());
    b.replace(&domain, Value::option_some(Value::Cell(b.clone())))
        .unwrap();
    assert!(weak_c.upgrade().is_none());
    root_b.with_values(|values| {
        let Value::Cell(alias) = &values[0] else {
            panic!("cell")
        };
        assert!(alias.ptr_eq(&b));
        assert!(matches!(
            *alias.read_snapshot(&domain).unwrap(),
            Value::OptionSome(_)
        ));
    });
    drop(b);
    drop(root_b);
    assert!(weak_b.upgrade().is_none());
}
#[test]
fn native_parking_forces_collection_and_keeps_published_roots() {
    let domain = domain(schedule(usize::MAX, usize::MAX, usize::MAX));
    let owner = domain.execution_roots().unwrap();
    let access = owner.enter().unwrap();
    let garbage = cycle(&domain);
    let garbage_weak = garbage.downgrade();
    drop(garbage);
    let live = cycle(&domain);
    let live_weak = live.downgrade();
    owner.publish(vec![Value::Cell(live)]);
    let wait = domain.publish_wait().unwrap();
    assert!(garbage_weak.upgrade().is_none());
    assert!(live_weak.upgrade().is_some());
    drop(wait);
    owner.publish(vec![]);
    let wait = domain.publish_wait().unwrap();
    assert!(live_weak.upgrade().is_none());
    drop(wait);
    drop(access);
}
#[test]
fn failed_preparation_keeps_pressure_and_pending_work_for_retry() {
    struct Busy(AtomicBool);
    impl crate::TraceRoots for Busy {
        fn trace_roots(&self, _: &mut dyn FnMut(&Value)) -> Result<(), OwnershipError> {
            if self.0.load(Ordering::SeqCst) {
                Err(OwnershipError::Busy)
            } else {
                Ok(())
            }
        }
    }
    let domain = domain(schedule(1, usize::MAX, usize::MAX));
    let owner = domain.execution_roots().unwrap();
    let access = owner.enter().unwrap();
    let busy = Arc::new(Busy(AtomicBool::new(true)));
    let _roots = domain.register_roots(RootKind::Inspection, busy.clone());
    let cell = cycle(&domain);
    let weak = cell.downgrade();
    drop(cell);
    owner.publish(vec![]);
    assert_eq!(
        access.batch_safe_point(),
        MaintenanceOutcome::Busy(OwnershipError::Busy)
    );
    assert!(weak.upgrade().is_some());
    assert!(domain.maintenance_pending());
    busy.0.store(false, Ordering::SeqCst);
    assert!(matches!(
        access.safe_point(),
        MaintenanceOutcome::Serviced(_)
    ));
    assert!(weak.upgrade().is_none());
}
