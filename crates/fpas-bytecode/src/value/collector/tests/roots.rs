use super::*;
use crate::{DomainPhase, ReferenceRoot, VariableReference};
#[test]
fn every_registered_holder_kind_preserves_its_cycle_until_release() {
    for kind in [
        RootKind::Host,
        RootKind::Native,
        RootKind::Inspection,
        RootKind::Evaluation,
        RootKind::Execution,
        RootKind::Globals,
        RootKind::Selection,
    ] {
        let domain = domain();
        let cell = cycle(&domain);
        let weak = cell.downgrade();
        let root = domain.root_values(kind, vec![Value::Cell(cell)]);
        collect(&domain);
        assert!(weak.upgrade().is_some());
        drop(root);
        collect(&domain);
        assert!(weak.upgrade().is_none());
    }
}
#[test]
fn a_var_reference_marks_the_cell_behind_its_root() {
    let domain = domain();
    let cell = cycle(&domain);
    let weak = cell.downgrade();
    let value = Value::Reference(Arc::new(VariableReference {
        root: ReferenceRoot::Cell(cell),
        path: vec![],
    }));
    let root = domain.root_values(RootKind::Native, vec![value]);
    collect(&domain);
    assert!(weak.upgrade().is_some());
    drop(root);
    collect(&domain);
    assert!(weak.upgrade().is_none());
}
#[test]
fn retired_independent_roots_and_edge_removing_mutations_reclaim_separately() {
    let (domain, session) = MemoryDomain::new_session();
    domain.set_maintenance(
        Arc::new(Collector(CollectionLimits::default())),
        MaintenancePolicy::Explicit,
    );
    let a = cycle(&domain);
    let wa = a.downgrade();
    let b = cycle(&domain);
    let c = cycle(&domain);
    let wc = c.downgrade();
    b.replace(
        &domain,
        Value::Array(vec![Value::Cell(b.clone()), Value::Cell(c)].into()),
    )
    .unwrap();
    let a = domain.root_values(RootKind::Host, vec![Value::Cell(a)]);
    let b_root = domain.root_values(RootKind::Host, vec![Value::Cell(b.clone())]);
    let identity = b.identity();
    drop(session);
    assert_eq!(domain.phase(), DomainPhase::Retired);
    collect(&domain);
    drop(a);
    collect(&domain);
    assert!(wa.upgrade().is_none());
    assert!(wc.upgrade().is_some());
    assert_eq!(b.identity(), identity);
    b.replace(
        &domain,
        Value::Array(vec![Value::Cell(b.clone()), Value::Integer(7)].into()),
    )
    .unwrap();
    collect(&domain);
    assert!(wc.upgrade().is_none());
    assert_eq!(b.identity(), identity);
    let Value::Array(contents) = b.read(&domain).unwrap() else {
        panic!("surviving value");
    };
    assert_eq!(contents[1], Value::Integer(7));
    drop(contents);
    drop(b_root);
    drop(b);
    collect(&domain);
    assert_eq!(domain.collection_stats().live_cells, 0);
}
#[test]
fn acyclic_values_release_immediately_without_a_collection_request() {
    let domain = domain();
    let cell = domain.new_cell(Value::Integer(1)).unwrap();
    let weak = cell.downgrade();
    drop(cell);
    assert!(weak.upgrade().is_none());
    let stats = domain.collection_stats();
    assert_eq!(stats.allocated_cells, 1);
    assert_eq!(stats.released_cells, 1);
    assert_eq!(stats.completed_passes, 0);
}
#[test]
fn bookkeeping_candidates_are_not_roots() {
    let domain = domain();
    let cell = cycle(&domain);
    let weak = cell.downgrade();
    // Administrative observations deliberately retain an unreachable storage object.
    let candidates = domain.live_cells();
    drop(cell);
    collect(&domain);
    assert_eq!(candidates[0].read(&domain).unwrap(), Value::Unit);
    assert_eq!(domain.collection_stats().cleared_cells, 1);
    collect(&domain);
    assert_eq!(
        domain.collection_stats().cleared_cells,
        1,
        "already empty payloads are unchanged"
    );
    drop(candidates);
    assert!(weak.upgrade().is_none());
}
