//! Retained selections validate before entry and isolate aggregate snapshots.

use super::{fixture::*, *};

#[test]
fn selected_index_is_frozen_and_snapshot_remains_unchanged() {
    let mut worker = invocation(
        vec![
            abc(Opcode::ReserveReference, 1, 0, 0),
            abc(Opcode::SelectReferenceIndex, 1, 1, 2),
            abx(Opcode::LoadConstant, 2, 0),
            abc_aux(Opcode::CallDirect, NO_REGISTER, 1, 1, 1),
            return_unit(),
        ],
        vec![(set_nine(), 2, vec![0])],
        3,
    );
    let snapshot = Value::Array(vec![Value::Integer(1), Value::Integer(2)].into());
    let cell = storage(&mut worker, snapshot.clone());
    worker.store_register(2, Value::Integer(0)).unwrap();
    run(&mut worker).unwrap();
    assert_eq!(
        worker.hosted.references.read(&cell).unwrap(),
        Value::Array(vec![Value::Integer(9), Value::Integer(2)].into())
    );
    assert_eq!(
        snapshot,
        Value::Array(vec![Value::Integer(1), Value::Integer(2)].into())
    );
}

#[test]
fn invalid_selection_releases_reservation_before_any_callee_write() {
    for index in [i64::MIN, -1, 1, i64::MAX] {
        let mut worker = invocation(
            vec![
                abc(Opcode::ReserveReference, 1, 0, 0),
                abc(Opcode::SelectReferenceIndex, 1, 1, 2),
                abc_aux(Opcode::CallDirect, NO_REGISTER, 1, 1, 1),
                return_unit(),
            ],
            vec![(set_nine(), 2, vec![0])],
            3,
        );
        let cell = storage(&mut worker, Value::Array(vec![Value::Integer(1)].into()));
        worker.store_register(2, Value::Integer(index)).unwrap();
        assert_eq!(
            run(&mut worker).unwrap_err().code,
            fpas_diagnostics::codes::RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS
        );
        worker
            .hosted
            .references
            .write(&cell, Value::Integer(7))
            .unwrap();
    }
}

#[test]
fn later_argument_mutation_through_alias_is_blocked_while_reserved() {
    let mut worker = invocation(
        vec![
            abc(Opcode::ReserveReference, 1, 0, 0),
            abx(Opcode::LoadConstant, 2, 0),
            abc(Opcode::CellWrite, 0, 2, 0),
            return_unit(),
        ],
        Vec::new(),
        3,
    );
    let cell = storage(&mut worker, Value::Integer(1));
    assert_eq!(
        run(&mut worker).unwrap_err().code,
        RUNTIME_STORAGE_REFERENCE_CONFLICT
    );
    assert_eq!(
        worker.hosted.references.read(&cell).unwrap(),
        Value::Integer(1)
    );
}
