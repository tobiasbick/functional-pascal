use super::*;
use crate::TraceRoots;
#[test]
fn scratch_bounds_abort_a_whole_pass_before_any_clear() {
    for limits in [
        CollectionLimits {
            max_cells: 1,
            max_graph_nodes: usize::MAX,
        },
        CollectionLimits {
            max_cells: usize::MAX,
            max_graph_nodes: 0,
        },
        CollectionLimits {
            max_cells: usize::MAX,
            max_graph_nodes: 1,
        },
    ] {
        let domain = domain();
        let a = cycle(&domain);
        let b = cycle(&domain);
        let wa = a.downgrade();
        let wb = b.downgrade();
        let root = domain.root_values(RootKind::Host, vec![Value::Cell(b)]);
        drop(a);
        domain.set_maintenance(Arc::new(Collector(limits)), MaintenancePolicy::Explicit);
        assert_eq!(
            domain.request_explicit_maintenance(),
            MaintenanceOutcome::Busy(OwnershipError::ScratchUnavailable)
        );
        assert!(wa.upgrade().is_some());
        assert!(wb.upgrade().is_some());
        assert_eq!(domain.collection_stats().cleared_cells, 0);
        // Cancel neither pending work nor existing roots when replacing the handler.
        domain.set_maintenance(
            Arc::new(Collector(CollectionLimits::default())),
            MaintenancePolicy::Explicit,
        );
        collect(&domain);
        assert!(wa.upgrade().is_none());
        assert!(wb.upgrade().is_some());
        drop(root);
        collect(&domain);
        assert!(wb.upgrade().is_none());
    }
}
#[test]
fn busy_or_poisoned_unreachable_cells_leave_other_candidates_unchanged() {
    for poisoned in [false, true] {
        let domain = domain();
        let a = cycle(&domain);
        let b = if poisoned {
            domain.new_cell(Value::Integer(2)).unwrap()
        } else {
            cycle(&domain)
        };
        let wa = a.downgrade();
        drop(a);
        if poisoned {
            let _ = std::panic::catch_unwind(|| {
                let _lock = b.lock(false).unwrap();
                panic!("poison cell");
            });
            assert_eq!(
                domain.request_explicit_maintenance(),
                MaintenanceOutcome::Busy(OwnershipError::Poisoned)
            );
        } else {
            let lock = b.lock(false).unwrap();
            assert_eq!(
                domain.request_explicit_maintenance(),
                MaintenanceOutcome::Busy(OwnershipError::Busy)
            );
            assert!(wa.upgrade().is_some());
            assert_eq!(domain.collection_stats().cleared_cells, 0);
            drop(lock);
        }
        assert!(wa.upgrade().is_some());
        assert_eq!(domain.collection_stats().cleared_cells, 0);
        drop(b);
        collect(&domain);
        assert!(wa.upgrade().is_none());
    }
}
#[test]
fn an_incomplete_registered_provider_never_authorizes_clearing() {
    struct Busy;
    impl TraceRoots for Busy {
        fn trace_roots(&self, _: &mut dyn FnMut(&Value)) -> Result<(), OwnershipError> {
            Err(OwnershipError::Busy)
        }
    }
    let domain = domain();
    let cell = cycle(&domain);
    let weak = cell.downgrade();
    drop(cell);
    let busy = domain.register_roots(RootKind::Native, Arc::new(Busy));
    assert_eq!(
        domain.request_explicit_maintenance(),
        MaintenanceOutcome::Busy(OwnershipError::Busy)
    );
    assert!(weak.upgrade().is_some());
    assert_eq!(domain.collection_stats().cleared_cells, 0);
    drop(busy);
    collect(&domain);
    assert!(weak.upgrade().is_none());
}
#[test]
fn direct_reclamation_without_exclusive_maintenance_is_rejected() {
    let domain = domain();
    let cell = cycle(&domain);
    let weak = cell.downgrade();
    drop(cell);
    assert_eq!(
        domain.reclaim_unreachable(&[], CollectionLimits::default()),
        Err(OwnershipError::Busy)
    );
    assert!(weak.upgrade().is_some());
    collect(&domain);
    assert!(weak.upgrade().is_none());
}
#[test]
fn foreign_roots_abort_before_traversing_or_clearing_either_domain() {
    let domain = domain();
    let foreign = MemoryDomain::new();
    let a = cycle(&domain);
    let wa = a.downgrade();
    drop(a);
    let b = cycle(&foreign);
    let wb = b.downgrade();
    let root = domain.root_values(RootKind::Host, vec![Value::Cell(b)]);
    assert_eq!(
        domain.request_explicit_maintenance(),
        MaintenanceOutcome::Busy(OwnershipError::ForeignDomain)
    );
    assert!(wa.upgrade().is_some());
    assert!(wb.upgrade().is_some());
    assert_eq!(domain.collection_stats().cleared_cells, 0);
    drop(root);
    collect(&domain);
    assert!(wa.upgrade().is_none());
    let b = wb.upgrade().unwrap();
    b.replace(&foreign, Value::Unit).unwrap();
}
