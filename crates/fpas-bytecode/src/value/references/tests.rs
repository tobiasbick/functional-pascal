use std::sync::{Arc, Mutex};

use super::{ReferenceError, ReferenceRegistry};
use crate::Value;

fn cell(value: i64) -> Arc<Mutex<Value>> {
    Arc::new(Mutex::new(Value::Integer(value)))
}

#[test]
fn reservation_allows_snapshots_but_rejects_alias_mutation() {
    let registry = ReferenceRegistry::default();
    let root = cell(1);
    let borrowed = registry.reserve(&root).unwrap();
    assert_eq!(registry.read(&root).unwrap(), Value::Integer(1));
    assert_eq!(borrowed.read().unwrap(), Value::Integer(1));
    assert_eq!(
        registry.write(&root, Value::Integer(2)),
        Err(ReferenceError::Reserved)
    );
    assert_eq!(
        borrowed.write(Value::Integer(2)),
        Err(ReferenceError::Reserved)
    );
}

#[test]
fn two_names_for_one_root_cannot_create_two_var_arguments() {
    let registry = ReferenceRegistry::default();
    let root = cell(1);
    let alias = Arc::clone(&root);
    let _first = registry.reserve(&root).unwrap();
    assert_eq!(
        registry.reserve(&alias).unwrap_err(),
        ReferenceError::Reserved
    );
}

#[test]
fn distinct_roots_can_be_reserved_and_activated_together() {
    let registry = ReferenceRegistry::default();
    let left = cell(1);
    let right = cell(2);
    let first = registry.reserve(&left).unwrap();
    let second = registry.reserve(&right).unwrap();
    first.activate().unwrap();
    second.activate().unwrap();
    first.write(Value::Integer(3)).unwrap();
    second.write(Value::Integer(4)).unwrap();
    assert_eq!(first.read().unwrap(), Value::Integer(3));
    assert_eq!(second.read().unwrap(), Value::Integer(4));
}

#[test]
fn active_reference_excludes_other_alias_reads_and_writes() {
    let registry = ReferenceRegistry::default();
    let root = cell(1);
    let borrowed = registry.reserve(&root).unwrap();
    borrowed.activate().unwrap();
    assert_eq!(registry.read(&root), Err(ReferenceError::Exclusive));
    assert_eq!(
        registry.write(&root, Value::Integer(2)),
        Err(ReferenceError::Exclusive)
    );
    assert_eq!(
        registry.reserve(&root).unwrap_err(),
        ReferenceError::Exclusive
    );
}

#[test]
fn writes_are_retained_when_reference_scope_returns_an_error() {
    let registry = ReferenceRegistry::default();
    let root = cell(1);
    let operation = || -> Result<(), &'static str> {
        let borrowed = registry.reserve(&root).unwrap();
        borrowed.activate().unwrap();
        borrowed.write(Value::Integer(7)).unwrap();
        Err("propagated error")
    };
    assert_eq!(operation(), Err("propagated error"));
    assert_eq!(registry.read(&root).unwrap(), Value::Integer(7));
}

#[test]
fn unwinding_releases_authority_without_rolling_back_mutation() {
    let registry = ReferenceRegistry::default();
    let root = cell(1);
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let borrowed = registry.reserve(&root).unwrap();
        borrowed.activate().unwrap();
        borrowed.write(Value::Integer(8)).unwrap();
        panic!("test unwind");
    }));
    assert!(outcome.is_err());
    assert_eq!(registry.read(&root).unwrap(), Value::Integer(8));
}

#[test]
fn a_forwarded_reservation_allows_only_its_parent_snapshot() {
    let registry = ReferenceRegistry::default();
    let root = cell(1);
    let parent = registry.reserve(&root).unwrap();
    parent.activate().unwrap();
    let child = parent.reborrow().unwrap();
    assert_eq!(parent.read().unwrap(), Value::Integer(1));
    assert_eq!(child.read().unwrap(), Value::Integer(1));
    assert_eq!(
        parent.write(Value::Integer(2)),
        Err(ReferenceError::Reserved)
    );
    assert_eq!(registry.read(&root), Err(ReferenceError::Exclusive));
    assert_eq!(parent.reborrow().unwrap_err(), ReferenceError::Suspended);
}

#[test]
fn active_reborrow_suspends_parent_until_return() {
    let registry = ReferenceRegistry::default();
    let root = cell(1);
    let parent = registry.reserve(&root).unwrap();
    parent.activate().unwrap();
    let child = parent.reborrow().unwrap();
    child.activate().unwrap();
    child.write(Value::Integer(3)).unwrap();
    assert_eq!(parent.read(), Err(ReferenceError::Suspended));
    assert_eq!(
        parent.write(Value::Integer(4)),
        Err(ReferenceError::Suspended)
    );
    child.release().unwrap();
    assert_eq!(parent.read().unwrap(), Value::Integer(3));
    parent.write(Value::Integer(5)).unwrap();
    parent.release().unwrap();
    assert_eq!(registry.read(&root).unwrap(), Value::Integer(5));
}

#[test]
fn nested_reborrows_restore_each_parent_in_order() {
    let registry = ReferenceRegistry::default();
    let root = cell(1);
    let parent = registry.reserve(&root).unwrap();
    parent.activate().unwrap();
    let child = parent.reborrow().unwrap();
    child.activate().unwrap();
    let grandchild = child.reborrow().unwrap();
    grandchild.activate().unwrap();
    assert_eq!(parent.read(), Err(ReferenceError::Suspended));
    assert_eq!(child.read(), Err(ReferenceError::Suspended));
    grandchild.write(Value::Integer(6)).unwrap();
    drop(grandchild);
    assert_eq!(child.read().unwrap(), Value::Integer(6));
    drop(child);
    assert_eq!(parent.read().unwrap(), Value::Integer(6));
}

#[test]
fn a_child_keeps_its_parent_alive_after_register_copies_are_dropped() {
    let registry = ReferenceRegistry::default();
    let root = cell(1);
    let parent = registry.reserve(&root).unwrap();
    parent.activate().unwrap();
    let child = parent.reborrow().unwrap();
    child.activate().unwrap();
    drop(parent);
    child.write(Value::Integer(2)).unwrap();
    drop(child);
    assert_eq!(registry.read(&root).unwrap(), Value::Integer(2));
}

#[test]
fn explicit_release_invalidates_retained_register_copies() {
    let registry = ReferenceRegistry::default();
    let root = cell(1);
    let borrowed = registry.reserve(&root).unwrap();
    let retained = borrowed.clone();
    borrowed.activate().unwrap();
    borrowed.release().unwrap();
    retained.release().unwrap();
    assert_eq!(retained.read(), Err(ReferenceError::Released));
    assert_eq!(
        retained.write(Value::Integer(2)),
        Err(ReferenceError::Released)
    );
    assert_eq!(retained.activate(), Err(ReferenceError::Released));
    let replacement = registry.reserve(&root).unwrap();
    replacement.activate().unwrap();
    assert_eq!(retained.read(), Err(ReferenceError::Released));
    assert_eq!(replacement.read().unwrap(), Value::Integer(1));
    assert_eq!(retained.activate(), Err(ReferenceError::Released));
    assert_eq!(retained.reborrow().unwrap_err(), ReferenceError::Released);
}

#[test]
fn parent_cannot_release_while_child_owns_authority() {
    let registry = ReferenceRegistry::default();
    let root = cell(1);
    let parent = registry.reserve(&root).unwrap();
    parent.activate().unwrap();
    let child = parent.reborrow().unwrap();
    assert_eq!(parent.release(), Err(ReferenceError::Suspended));
    child.release().unwrap();
    parent.release().unwrap();
    registry.write(&root, Value::Integer(2)).unwrap();
}

#[test]
fn forwarding_requires_active_authority() {
    let registry = ReferenceRegistry::default();
    let root = cell(1);
    let reserved = registry.reserve(&root).unwrap();
    assert_eq!(reserved.reborrow().unwrap_err(), ReferenceError::NotActive);
    reserved.activate().unwrap();
    assert_eq!(reserved.activate(), Err(ReferenceError::AlreadyActive));
}

#[test]
fn reservation_snapshot_isolates_nested_value_storage() {
    let registry = ReferenceRegistry::default();
    let root = Arc::new(Mutex::new(Value::Array(
        vec![Value::Array(vec![Value::Integer(1)].into())].into(),
    )));
    let borrowed = registry.reserve(&root).unwrap();
    let snapshot = registry.read(&root).unwrap();
    borrowed.activate().unwrap();
    let Value::Array(mut outer) = borrowed.read().unwrap() else {
        panic!("array root")
    };
    let Value::Array(inner) = &mut outer[0] else {
        panic!("nested array")
    };
    inner[0] = Value::Integer(9);
    borrowed.write(Value::Array(outer)).unwrap();
    assert_eq!(snapshot.to_string(), "[[1]]");
    assert_eq!(borrowed.read().unwrap().to_string(), "[[9]]");
}

#[test]
fn cloned_registry_enforces_authority_across_worker_threads() {
    let registry = ReferenceRegistry::default();
    let root = cell(1);
    let borrowed = registry.reserve(&root).unwrap();
    borrowed.activate().unwrap();
    let other_registry = registry.clone();
    let other_root = Arc::clone(&root);
    let result = std::thread::spawn(move || other_registry.write(&other_root, Value::Integer(2)))
        .join()
        .unwrap();
    assert_eq!(result, Err(ReferenceError::Exclusive));
    borrowed.release().unwrap();
    registry.write(&root, Value::Integer(3)).unwrap();
}

#[test]
fn abandoned_argument_reservation_releases_the_root() {
    let registry = ReferenceRegistry::default();
    let root = cell(1);
    drop(registry.reserve(&root).unwrap());
    registry.write(&root, Value::Integer(2)).unwrap();
    assert_eq!(registry.read(&root).unwrap(), Value::Integer(2));
}

#[test]
fn deep_reborrow_cleanup_does_not_recurse_through_parent_destructors() {
    let registry = ReferenceRegistry::default();
    let root = cell(1);
    let mut current = registry.reserve(&root).unwrap();
    current.activate().unwrap();
    for _ in 0..4096 {
        let child = current.reborrow().unwrap();
        child.activate().unwrap();
        current = child;
    }
    current.write(Value::Integer(2)).unwrap();
    drop(current);
    assert_eq!(registry.read(&root).unwrap(), Value::Integer(2));
}
