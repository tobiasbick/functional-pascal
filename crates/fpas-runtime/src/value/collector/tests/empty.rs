//! Empty passes retain root preparation, ownership checks and request accounting.
use super::*;
use crate::value::graph::tests::target;
use crate::{ReferenceRoot, TraceRoots, VariableReference};
use std::sync::atomic::{AtomicBool, Ordering};

fn global_reference(domain: &MemoryDomain) -> Value {
    Value::Reference(Arc::new(VariableReference {
        root: ReferenceRoot::Global {
            slot: 0,
            owner: domain.id().clone(),
        },
        path: vec![],
    }))
}

#[test]
fn empty_passes_acknowledge_roots_and_later_cell_allocations_still_collect() {
    let domain = domain();
    drop(domain.new_cell(Value::Integer(1)).unwrap());
    let array = Value::Array(vec![Value::Integer(2), Value::Integer(3)].into());
    let root = domain.root_values(
        RootKind::Host,
        vec![
            Value::option_some(Value::dict(vec![(Value::Integer(1), array.clone())])),
            Value::task_owned_function(target(), vec![array], domain.id().clone()),
            global_reference(&domain),
        ],
    );
    collect(&domain);
    let stats = domain.collection_stats();
    assert_eq!(stats.live_cells, 0);
    assert_eq!(stats.released_cells, 1);
    assert_eq!(stats.completed_passes, 1);
    assert_eq!(stats.aborted_passes, 0);
    assert!(!domain.maintenance_pending());
    assert_eq!(
        domain.serviced_generation(),
        domain.maintenance_generation()
    );

    let cell = cycle(&domain);
    let weak = cell.downgrade();
    let live = domain.root_values(RootKind::Host, vec![Value::Cell(cell)]);
    collect(&domain);
    assert!(weak.upgrade().is_some());
    drop(live);
    collect(&domain);
    assert!(weak.upgrade().is_none());
    assert_eq!(domain.collection_stats().cleared_cells, 1);
    drop(root);
    collect(&domain);
    assert!(!domain.maintenance_pending());
}

#[test]
fn empty_passes_preserve_finite_graph_bounds() {
    for max_graph_nodes in 0..=4 {
        let domain = domain();
        let _root = domain.root_values(
            RootKind::Host,
            vec![Value::option_some(Value::Array(
                vec![Value::Integer(1), Value::Integer(2)].into(),
            ))],
        );
        domain.set_maintenance(
            Arc::new(Collector(CollectionLimits {
                max_cells: 0,
                max_graph_nodes,
            })),
            MaintenancePolicy::Explicit,
        );
        let outcome = domain.request_explicit_maintenance();
        if max_graph_nodes < 4 {
            assert_eq!(
                outcome,
                MaintenanceOutcome::Busy(OwnershipError::ScratchUnavailable)
            );
            assert!(domain.maintenance_pending());
            assert_eq!(domain.serviced_generation(), 0);
            assert_eq!(domain.collection_stats().aborted_passes, 1);
            assert_eq!(domain.collection_stats().completed_passes, 0);
        } else {
            assert!(matches!(outcome, MaintenanceOutcome::Serviced(_)));
            assert_eq!(domain.collection_stats().completed_passes, 1);
            assert!(!domain.maintenance_pending());
            assert_eq!(
                domain.serviced_generation(),
                domain.maintenance_generation()
            );
        }
    }
}

#[test]
fn empty_domains_still_require_complete_registered_providers() {
    struct Provider(AtomicBool);
    impl TraceRoots for Provider {
        fn trace_roots(&self, visit: &mut dyn FnMut(&Value)) -> Result<(), OwnershipError> {
            visit(&Value::Integer(1));
            if self.0.load(Ordering::SeqCst) {
                Err(OwnershipError::Busy)
            } else {
                Ok(())
            }
        }
    }
    let domain = domain();
    let provider = Arc::new(Provider(AtomicBool::new(true)));
    let _root = domain.register_roots(RootKind::Native, provider.clone());
    assert_eq!(
        domain.request_explicit_maintenance(),
        MaintenanceOutcome::Busy(OwnershipError::Busy)
    );
    assert!(domain.maintenance_pending());
    assert_eq!(domain.serviced_generation(), 0);
    assert_eq!(domain.collection_stats().completed_passes, 0);
    provider.0.store(false, Ordering::SeqCst);
    assert!(matches!(
        domain.try_service_maintenance(),
        MaintenanceOutcome::Serviced(_)
    ));
    assert!(!domain.maintenance_pending());
    assert_eq!(
        domain.serviced_generation(),
        domain.maintenance_generation()
    );
    assert_eq!(domain.collection_stats().completed_passes, 1);
}

#[test]
fn empty_domains_reject_foreign_callable_and_global_reference_owners() {
    let foreign = MemoryDomain::new();
    for value in [
        Value::task_owned_function(target(), vec![], foreign.id().clone()),
        global_reference(&foreign),
    ] {
        let domain = domain();
        let root = domain.root_values(
            RootKind::Host,
            vec![Value::Array(vec![Value::Integer(1), value].into())],
        );
        assert_eq!(
            domain.request_explicit_maintenance(),
            MaintenanceOutcome::Busy(OwnershipError::ForeignDomain)
        );
        assert!(domain.maintenance_pending());
        assert_eq!(domain.collection_stats().live_cells, 0);
        assert_eq!(domain.collection_stats().completed_passes, 0);
        drop(root);
        collect(&domain);
        assert!(!domain.maintenance_pending());
    }
}

#[test]
fn empty_domains_cannot_bypass_exclusive_collection() {
    let domain = domain();
    assert_eq!(
        domain.reclaim_unreachable(&[], CollectionLimits::default()),
        Err(OwnershipError::Busy)
    );
    assert_eq!(domain.collection_stats().completed_passes, 0);
    collect(&domain);
    assert_eq!(domain.collection_stats().completed_passes, 1);
}
