//! Reservation, exclusivity, forwarding and atomic selected updates.

use super::*;

#[test]
fn reservations_cover_the_entire_root_even_for_different_elements() {
    let registry = ReferenceRegistry::default();
    let root = cell(Value::Array(
        vec![Value::Integer(1), Value::Integer(2)].into(),
    ));
    let selected = SelectedReference::root(registry.reserve(&root).unwrap())
        .project(index(0))
        .unwrap();
    assert_eq!(
        registry.reserve(&root).unwrap_err(),
        ReferenceError::Reserved
    );
    assert_eq!(
        selected.write(Value::Integer(3)),
        Err(ReferencePathError::Access(ReferenceError::Reserved))
    );
    assert_eq!(
        registry.write(&root, Value::Array(vec![].into())),
        Err(ReferenceError::Reserved)
    );
    assert_eq!(
        registry.read(&root).unwrap(),
        Value::Array(vec![Value::Integer(1), Value::Integer(2)].into())
    );
    selected.activate().unwrap();
    assert_eq!(registry.read(&root), Err(ReferenceError::Exclusive));
    assert_eq!(
        registry.reserve(&root).unwrap_err(),
        ReferenceError::Exclusive
    );
}

#[test]
fn forwarding_keeps_the_path_and_restores_parent_access_without_rollback() {
    let registry = ReferenceRegistry::default();
    let root = cell(record(vec![Value::Array(vec![Value::Integer(1)].into())]));
    let parent = SelectedReference::root(registry.reserve(&root).unwrap())
        .project(field(0))
        .unwrap();
    parent.activate().unwrap();
    let child = parent.reborrow().unwrap().project(index(0)).unwrap();
    assert_eq!(
        parent.read().unwrap(),
        Value::Array(vec![Value::Integer(1)].into())
    );
    assert_eq!(
        parent.write(Value::Array(vec![].into())),
        Err(ReferencePathError::Access(ReferenceError::Reserved))
    );
    child.activate().unwrap();
    assert_eq!(
        parent.read(),
        Err(ReferencePathError::Access(ReferenceError::Suspended))
    );
    child.write(Value::Integer(42)).unwrap();
    child.release().unwrap();
    assert_eq!(
        parent.read().unwrap(),
        Value::Array(vec![Value::Integer(42)].into())
    );
    parent.release().unwrap();
    assert_eq!(
        read(&registry.read(&root).unwrap(), &[field(0), index(0)]).unwrap(),
        &Value::Integer(42)
    );
}

#[test]
fn released_selected_clones_cannot_read_write_project_or_forward() {
    let registry = ReferenceRegistry::default();
    let root = cell(Value::Array(vec![Value::Integer(1)].into()));
    let selected = SelectedReference::root(registry.reserve(&root).unwrap());
    let retained = selected.clone();
    selected.activate().unwrap();
    selected.release().unwrap();
    assert_eq!(
        retained.read(),
        Err(ReferencePathError::Access(ReferenceError::Released))
    );
    assert_eq!(
        retained.write(Value::Integer(2)),
        Err(ReferencePathError::Access(ReferenceError::Released))
    );
    assert_eq!(
        retained.project(index(0)).unwrap_err(),
        ReferencePathError::Access(ReferenceError::Released)
    );
    assert_eq!(retained.reborrow().unwrap_err(), ReferenceError::Released);
    assert_eq!(
        registry.read(&root).unwrap(),
        Value::Array(vec![Value::Integer(1)].into())
    );
}

#[test]
fn simultaneous_authorized_element_updates_do_not_overwrite_each_other() {
    let registry = ReferenceRegistry::default();
    let root = cell(Value::Array(
        vec![Value::Integer(0), Value::Integer(0)].into(),
    ));
    let borrowed = SelectedReference::root(registry.reserve(&root).unwrap());
    let left = borrowed.project(index(0)).unwrap();
    let right = borrowed.project(index(1)).unwrap();
    borrowed.activate().unwrap();
    let barrier = Arc::new(std::sync::Barrier::new(3));
    let left_barrier = Arc::clone(&barrier);
    let left_thread = std::thread::spawn(move || {
        left_barrier.wait();
        for value in 1..=256 {
            left.write(Value::Integer(value)).unwrap();
        }
    });
    let right_barrier = Arc::clone(&barrier);
    let right_thread = std::thread::spawn(move || {
        right_barrier.wait();
        for value in 1..=256 {
            right.write(Value::Integer(value)).unwrap();
        }
    });
    barrier.wait();
    left_thread.join().unwrap();
    right_thread.join().unwrap();
    assert_eq!(
        borrowed.read().unwrap(),
        Value::Array(vec![Value::Integer(256), Value::Integer(256)].into())
    );
}
