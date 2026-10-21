use super::*;
use crate::{ReferenceRoot, VariableReference};

fn deep(mut value: Value) -> Value {
    for depth in 0..30_000 {
        value = match depth % 3 {
            0 => Value::option_some(value),
            1 => Value::result_ok(value),
            _ => Value::result_error(value),
        };
    }
    value
}

fn small_stack(run: fn()) {
    std::thread::Builder::new()
        .stack_size(128 * 1024)
        .spawn(run)
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn rejected_construction_releases_deep_foreign_and_reference_payloads() {
    small_stack(|| {
        let owner = MemoryDomain::new();
        let foreign = MemoryDomain::new();
        let foreign_cell = foreign.new_cell(Value::Unit).unwrap();
        let weak = foreign_cell.downgrade();
        assert!(matches!(
            owner.new_cell(deep(Value::Cell(foreign_cell))),
            Err(OwnershipError::ForeignDomain)
        ));
        assert!(weak.upgrade().is_none());
        assert_eq!(owner.registered_count(), 0);
        assert_eq!(foreign.registered_count(), 0);

        let reference = Value::Reference(Arc::new(VariableReference {
            root: ReferenceRoot::Global {
                slot: 0,
                owner: owner.id().clone(),
            },
            path: Vec::new(),
        }));
        assert!(matches!(
            owner.new_cell(deep(reference)),
            Err(OwnershipError::EscapingReference)
        ));
        assert_eq!(owner.registered_count(), 0);
    });
}

#[test]
fn rejected_replacement_releases_deep_input_before_and_after_validation() {
    small_stack(|| {
        let owner = MemoryDomain::new();
        let foreign = MemoryDomain::new();
        let cell = owner.new_cell(Value::Integer(7)).unwrap();
        assert_eq!(
            cell.replace(&foreign, deep(Value::Integer(1))),
            Err(OwnershipError::ForeignDomain)
        );
        assert_eq!(
            cell.try_replace(
                &owner,
                deep(Value::Cell(foreign.new_cell(Value::Unit).unwrap()))
            ),
            Err(OwnershipError::ForeignDomain)
        );
        assert_eq!(cell.read(&owner).unwrap(), Value::Integer(7));

        let guard = cell.0.value.lock().unwrap();
        assert_eq!(
            cell.try_replace(&owner, deep(Value::Unit)),
            Err(OwnershipError::Busy)
        );
        drop(guard);
        assert_eq!(cell.read(&owner).unwrap(), Value::Integer(7));

        let _ = std::panic::catch_unwind(|| {
            let _guard = cell.0.value.lock().unwrap();
            panic!("interrupt replacement storage");
        });
        assert_eq!(
            cell.replace(&owner, deep(Value::Unit)),
            Err(OwnershipError::Poisoned)
        );
    });
}

#[test]
fn rejected_update_releases_deep_transformed_graph_and_preserves_current_value() {
    small_stack(|| {
        let owner = MemoryDomain::new();
        let foreign = MemoryDomain::new();
        let cell = owner.new_cell(Value::Integer(1)).unwrap();
        let result = cell.update(&owner, |_| {
            Ok::<_, ()>(deep(Value::Cell(foreign.new_cell(Value::Unit).unwrap())))
        });
        assert!(matches!(
            result,
            Err(CellUpdateError::Ownership(OwnershipError::ForeignDomain))
        ));
        assert_eq!(cell.read(&owner).unwrap(), Value::Integer(1));

        let result = cell.update(&owner, |_| {
            cell.replace(&owner, Value::Integer(2)).unwrap();
            Ok::<_, ()>(deep(Value::Integer(3)))
        });
        assert!(matches!(
            result,
            Err(CellUpdateError::Ownership(
                OwnershipError::ConcurrentMutation
            ))
        ));
        assert_eq!(cell.read(&owner).unwrap(), Value::Integer(2));
    });
}

#[test]
fn unavailable_update_commit_releases_its_deep_candidate() {
    small_stack(|| {
        let owner = MemoryDomain::new();
        let cell = owner.new_cell(Value::Integer(1)).unwrap();
        let mut guard = None;
        let result = cell.try_update(&owner, |_| {
            guard = Some(cell.0.value.lock().unwrap());
            Ok::<_, ()>(deep(Value::Unit))
        });
        assert!(matches!(
            result,
            Err(CellUpdateError::Ownership(OwnershipError::Busy))
        ));
        drop(guard);
        assert_eq!(cell.read(&owner).unwrap(), Value::Integer(1));

        let result = cell.update(&owner, |_| {
            let _ = std::panic::catch_unwind(|| {
                let _guard = cell.0.value.lock().unwrap();
                panic!("interrupt update commit storage");
            });
            Ok::<_, ()>(deep(Value::Unit))
        });
        assert!(matches!(
            result,
            Err(CellUpdateError::Ownership(OwnershipError::Poisoned))
        ));
    });
}
