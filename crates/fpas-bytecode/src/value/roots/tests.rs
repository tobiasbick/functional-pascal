use super::*;

#[test]
fn independent_nonfinal_root_release_requests_maintenance() {
    let domain = MemoryDomain::new();
    let first = domain.new_cell(Value::Integer(1)).expect("first");
    let weak = first.downgrade();
    let a = domain.root_values(RootKind::Host, vec![Value::Cell(first)]);
    let b = domain.root_values(RootKind::Host, vec![Value::Integer(2)]);
    assert_eq!(domain.root_count(), 2);
    drop(a);
    assert_eq!(domain.maintenance_generation(), 1);
    assert_eq!(domain.root_count(), 1);
    assert!(weak.upgrade().is_none());
    assert_eq!(b.with_values(|values| values[0].clone()), Value::Integer(2));
}

#[test]
fn removal_inside_nested_access_keeps_root_until_outer_release() {
    let domain = MemoryDomain::new();
    let cell = domain.new_cell(Value::Integer(3)).expect("cell");
    let weak = cell.downgrade();
    let root = domain.root_values(RootKind::Inspection, vec![Value::Cell(cell)]);
    let outer = domain.access();
    let inner = domain.access();
    drop(root);
    drop(inner);
    assert_eq!(domain.root_count(), 1);
    assert_eq!(domain.maintenance_generation(), 0);
    assert!(weak.upgrade().is_some());
    drop(outer);
    assert_eq!(domain.root_count(), 0);
    assert_eq!(domain.maintenance_generation(), 1);
    assert!(weak.upgrade().is_none());
}

#[test]
fn surviving_root_retains_only_domain_and_registered_graph() {
    let domain = MemoryDomain::new();
    let weak_domain = Arc::downgrade(&domain.0);
    let root = domain.root_values(RootKind::Host, vec![Value::Integer(4)]);
    drop(domain);
    assert!(weak_domain.upgrade().is_some());
    let copy = root.clone();
    drop(root);
    assert_eq!(
        copy.with_values(|values| values[0].clone()),
        Value::Integer(4)
    );
    drop(copy);
    assert!(weak_domain.upgrade().is_none());
}

#[test]
fn replacement_publishes_new_roots_and_notifies_for_disconnected_edges() {
    let domain = MemoryDomain::new();
    let old = domain.new_cell(Value::Integer(5)).expect("old");
    let weak = old.downgrade();
    let root = domain.root_values(RootKind::Native, vec![Value::Cell(old)]);
    root.replace(vec![Value::Integer(6)]);
    assert!(weak.upgrade().is_none());
    assert_eq!(domain.maintenance_generation(), 1);
    let mut seen = Vec::new();
    domain
        .trace_registered_roots(&mut |kind, value| {
            assert_eq!(kind, RootKind::Native);
            seen.push(value.clone());
        })
        .expect("complete root view");
    assert_eq!(seen, [Value::Integer(6)]);
}

#[test]
fn external_access_serializes_without_a_global_gate() {
    let domain = MemoryDomain::new();
    let outer = domain.access();
    let (started, start) = std::sync::mpsc::channel();
    let (done, finish) = std::sync::mpsc::channel();
    let other = domain.clone();
    let thread = std::thread::spawn(move || {
        started.send(()).expect("started");
        let _guard = other.access();
        done.send(()).expect("done");
    });
    start.recv().expect("worker started");
    assert!(finish.try_recv().is_err());
    let unrelated = MemoryDomain::new();
    let _unrelated = unrelated.access();
    drop(outer);
    finish.recv().expect("access released");
    thread.join().expect("worker");
}

#[test]
fn dropping_an_independent_clone_requests_maintenance_before_the_final_clone() {
    let domain = MemoryDomain::new();
    let a = domain.root_values(RootKind::Host, vec![Value::Integer(7)]);
    let b = a.clone();
    assert_eq!(domain.root_count(), 2);
    let access = domain.access();
    drop(a);
    assert_eq!(domain.root_count(), 2);
    assert_eq!(domain.maintenance_generation(), 0);
    drop(access);
    assert_eq!(domain.root_count(), 1);
    assert_eq!(domain.maintenance_generation(), 1);
    assert_eq!(b.with_values(|values| values[0].clone()), Value::Integer(7));
}

#[test]
fn a_busy_provider_never_silently_disappears_from_the_root_view() {
    let domain = MemoryDomain::new();
    let roots = domain.root_values(RootKind::Native, vec![Value::Unit]);
    roots.with_values(|_| {
        assert_eq!(
            domain.trace_registered_roots(&mut |_, _| {}),
            Err(OwnershipError::Busy)
        );
    });
    assert!(domain.trace_registered_roots(&mut |_, _| {}).is_ok());
}
