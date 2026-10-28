use super::*;

#[test]
fn integer_writes_preserve_aliases_and_immutable_snapshots() {
    let domain = MemoryDomain::new();
    let cell = domain.new_cell(Value::Integer(0)).unwrap();
    let alias = cell.clone();
    for next in [i64::MIN, -1, 0, 1, i64::MAX] {
        let previous = cell.read_snapshot(&domain).unwrap();
        let expected = (*previous).clone();
        cell.replace_integer(&domain, next).unwrap();
        assert_eq!(*previous, expected);
        assert_eq!(alias.read(&domain).unwrap(), Value::Integer(next));
        drop(previous);
        // The alias shares the cell, but no longer shares its immutable payload.
        alias.replace_integer(&domain, !next).unwrap();
        assert_eq!(cell.read(&domain).unwrap(), Value::Integer(!next));
    }
}

#[test]
fn integer_replacement_releases_graphs_only_after_the_last_snapshot() {
    for retained in [false, true] {
        let domain = MemoryDomain::new();
        let child = domain
            .new_cell(Value::Str("payload".repeat(1024).into()))
            .unwrap();
        let weak = child.downgrade();
        let cell = domain
            .new_cell(Value::Array(vec![Value::Cell(child)].into()))
            .unwrap();
        let snapshot = retained.then(|| cell.read_snapshot(&domain).unwrap());
        cell.replace_integer(&domain, 42).unwrap();
        assert_eq!(cell.read(&domain).unwrap(), Value::Integer(42));
        assert_eq!(weak.upgrade().is_some(), retained);
        if let Some(snapshot) = &snapshot {
            let Value::Array(array) = &**snapshot else {
                panic!("snapshot must retain its array");
            };
            assert!(matches!(array[0], Value::Cell(_)));
        }
        drop(snapshot);
        assert!(weak.upgrade().is_none());
        assert_eq!(domain.registered_count(), 1);
        drop(cell);
        assert_eq!(domain.registered_count(), 0);
    }
}

#[test]
fn integer_replacement_drains_deep_payloads_without_recursion() {
    std::thread::Builder::new()
        .stack_size(128 * 1024)
        .spawn(|| {
            for retained in [false, true] {
                let domain = MemoryDomain::new();
                let child = domain.new_cell(Value::Unit).unwrap();
                let weak = child.downgrade();
                let mut value = Value::Cell(child);
                for _ in 0..30_000 {
                    value = Value::option_some(value);
                }
                let cell = domain.new_cell(value).unwrap();
                let snapshot = retained.then(|| cell.read_snapshot(&domain).unwrap());
                cell.replace_integer(&domain, 7).unwrap();
                assert_eq!(weak.upgrade().is_some(), retained);
                drop(snapshot);
                assert!(weak.upgrade().is_none());
                assert_eq!(cell.read(&domain).unwrap(), Value::Integer(7));
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn integer_writes_reject_foreign_and_poisoned_access_without_replacement() {
    let domain = MemoryDomain::new();
    let foreign = MemoryDomain::new();
    let cell = domain.new_cell(Value::Integer(17)).unwrap();
    assert_eq!(
        cell.replace_integer(&foreign, 42),
        Err(OwnershipError::ForeignDomain)
    );
    assert_eq!(cell.read(&domain).unwrap(), Value::Integer(17));
    let interrupted = std::panic::catch_unwind(|| {
        let _lock = cell.0.value.lock().unwrap();
        panic!("interrupt cell access");
    });
    assert!(interrupted.is_err());
    assert_eq!(
        cell.replace_integer(&domain, 42),
        Err(OwnershipError::Poisoned)
    );
    // Ownership failure still takes precedence over poisoned storage.
    assert_eq!(
        cell.replace_integer(&foreign, 42),
        Err(OwnershipError::ForeignDomain)
    );
    let slot = cell.0.value.lock().unwrap_err().into_inner();
    assert_eq!(**slot, Value::Integer(17));
}

#[test]
fn integer_writes_invalidate_an_in_progress_snapshot_update() {
    let domain = MemoryDomain::new();
    let cell = domain.new_cell(Value::Integer(1)).unwrap();
    let result = cell.update(&domain, |snapshot| {
        cell.replace_integer(&domain, 2).unwrap();
        assert_eq!(snapshot, &Value::Integer(1));
        Ok::<_, ()>(Value::Integer(3))
    });
    assert!(matches!(
        result,
        Err(CellUpdateError::Ownership(
            OwnershipError::ConcurrentMutation
        ))
    ));
    assert_eq!(cell.read(&domain).unwrap(), Value::Integer(2));
}
