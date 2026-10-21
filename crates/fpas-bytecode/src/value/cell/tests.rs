use super::*;
use crate::{ReferenceRoot, VariableReference};

#[test]
fn domain_identity_survives_cloning_worker_migration_and_retirement() {
    let original = MemoryDomain::new();
    let clone = original.clone();
    let cell = original.new_cell(Value::Integer(7)).unwrap();
    let identity = original.id().clone();
    assert_eq!(original.id(), clone.id());
    drop(original);
    assert_eq!(cell.read(&clone).unwrap(), Value::Integer(7));
    assert_eq!(cell.owner(), &identity);
    assert_ne!(&identity, MemoryDomain::new().id());
    std::thread::spawn(move || {
        cell.replace(&clone, Value::Integer(8)).unwrap();
        assert_eq!(cell.read(&clone).unwrap(), Value::Integer(8));
    })
    .join()
    .unwrap();
}

#[test]
fn foreign_reads_and_mixed_graph_writes_fail_without_replacement() {
    let owner = MemoryDomain::new();
    let foreign = MemoryDomain::new();
    let cell = owner.new_cell(Value::Integer(7)).unwrap();
    let foreign_cell = foreign.new_cell(Value::Integer(9)).unwrap();
    assert_eq!(cell.read(&foreign), Err(OwnershipError::ForeignDomain));
    assert_eq!(cell.try_read(&foreign), Err(OwnershipError::ForeignDomain));
    let mixed = Value::dict(vec![(
        Value::Integer(1),
        Value::option_some(Value::Array(vec![Value::Cell(foreign_cell)].into())),
    )]);
    assert_eq!(
        cell.replace(&owner, mixed),
        Err(OwnershipError::ForeignDomain)
    );
    assert_eq!(cell.read(&owner).unwrap(), Value::Integer(7));
}

#[test]
fn retained_storage_rejects_references_but_local_validation_allows_arguments() {
    let domain = MemoryDomain::new();
    let cell = domain.new_cell(Value::Integer(4)).unwrap();
    let reference = Value::Reference(Arc::new(VariableReference {
        root: ReferenceRoot::Cell(cell.clone()),
        path: Vec::new(),
    }));
    assert!(graph::validate_local(&domain, &reference).is_ok());
    assert!(matches!(
        domain.new_cell(Value::option_some(reference.clone())),
        Err(OwnershipError::EscapingReference)
    ));
    assert_eq!(
        cell.replace(&domain, reference),
        Err(OwnershipError::EscapingReference)
    );
    assert_eq!(cell.read(&domain).unwrap(), Value::Integer(4));
}

#[test]
fn transformations_fail_atomically_and_detect_intervening_replacement() {
    let owner = MemoryDomain::new();
    let other = MemoryDomain::new();
    let cell = owner.new_cell(Value::Integer(1)).unwrap();
    let failure = cell.update(&owner, |_| Err("invalid path"));
    assert!(matches!(
        failure,
        Err(CellUpdateError::Update("invalid path"))
    ));
    let failure = cell.update(&owner, |_| {
        Ok::<_, ()>(Value::Cell(other.new_cell(Value::Unit).unwrap()))
    });
    assert!(matches!(
        failure,
        Err(CellUpdateError::Ownership(OwnershipError::ForeignDomain))
    ));
    assert_eq!(cell.read(&owner).unwrap(), Value::Integer(1));

    let result = cell.update(&owner, |snapshot| {
        assert_eq!(snapshot, &Value::Integer(1));
        cell.replace(&owner, Value::Integer(2)).unwrap();
        Ok::<_, ()>(Value::Integer(3))
    });
    assert!(matches!(
        result,
        Err(CellUpdateError::Ownership(
            OwnershipError::ConcurrentMutation
        ))
    ));
    assert_eq!(cell.read(&owner).unwrap(), Value::Integer(2));
    cell.try_update(&owner, |_| Ok::<_, ()>(Value::Integer(4)))
        .unwrap();
    assert_eq!(cell.read(&owner).unwrap(), Value::Integer(4));
}

#[test]
fn snapshots_keep_prior_contents_without_preventing_replacement() {
    let domain = MemoryDomain::new();
    let cell = domain.new_cell(Value::Integer(1)).unwrap();
    let before = cell.read_snapshot(&domain).unwrap();
    cell.try_replace(&domain, Value::Integer(2)).unwrap();
    assert_eq!(*before, Value::Integer(1));
    assert_eq!(cell.read(&domain).unwrap(), Value::Integer(2));
}

#[test]
fn classification_and_nonblocking_access_report_busy_cells() {
    let domain = MemoryDomain::new();
    let cell = domain.new_cell(Value::Integer(1)).unwrap();
    let _guard = cell.0.value.lock().unwrap();
    assert_eq!(cell.try_read(&domain), Err(OwnershipError::Busy));
    assert_eq!(
        cell.try_replace(&domain, Value::Unit),
        Err(OwnershipError::Busy)
    );
    assert!(matches!(
        graph::classify_values(&[Value::Cell(cell.clone())]),
        Err(OwnershipError::Busy)
    ));
}

#[test]
fn interrupted_storage_is_unavailable_to_reads_and_writes() {
    let domain = MemoryDomain::new();
    let cell = domain.new_cell(Value::Integer(1)).unwrap();
    let interruption = std::panic::catch_unwind(|| {
        let _guard = cell.0.value.lock().unwrap();
        panic!("interrupt cell access");
    });
    assert!(interruption.is_err());
    assert_eq!(cell.read(&domain), Err(OwnershipError::Poisoned));
    assert_eq!(
        cell.try_replace(&domain, Value::Unit),
        Err(OwnershipError::Poisoned)
    );
}

#[test]
fn weak_registration_releases_acyclic_cells_and_dead_entries() {
    let domain = MemoryDomain::new();
    for _ in 0..1024 {
        let cell = domain.new_cell(Value::Integer(1)).unwrap();
        let weak = cell.downgrade();
        assert_eq!(domain.registered_count(), 1);
        assert!(domain.live_cells()[0].ptr_eq(&cell));
        drop(cell);
        assert!(weak.upgrade().is_none());
        assert_eq!(domain.registered_count(), 0);
    }
    let cell = domain.new_cell(Value::Unit).unwrap();
    let weak = Arc::downgrade(&domain.0);
    drop(domain);
    assert!(weak.upgrade().is_none());
    drop(cell);
}
