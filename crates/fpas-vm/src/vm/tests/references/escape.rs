//! Reservations cannot become retained user values or closure captures.

use super::{fixture::*, *};

#[test]
fn reference_retention_operations_reject_without_leaking_the_reservation() {
    for operation in [
        abc(Opcode::MakeSome, 2, 1, 0),
        abc(Opcode::MakeOk, 2, 1, 0),
        abc(Opcode::MakeArray, 2, 1, 1),
        abc(Opcode::MakeCell, 2, 1, 0),
        abc(Opcode::CellWrite, 0, 1, 0),
    ] {
        let mut worker = invocation(
            vec![
                abc(Opcode::ReserveReference, 1, 0, 0),
                operation,
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
}

#[test]
fn var_parameter_rejects_plain_value_before_entry() {
    let mut worker = invocation(
        vec![
            abc(Opcode::ReserveReference, 1, 0, 0),
            abc_aux(Opcode::CallDirect, NO_REGISTER, 1, 1, 1),
            return_unit(),
        ],
        vec![(set_nine(), 2, vec![0])],
        2,
    );
    let cell = storage(&mut worker, Value::Integer(1));
    worker.store_register(1, Value::Integer(4)).unwrap();
    worker.ip = 1;
    assert_eq!(
        run(&mut worker).unwrap_err().code,
        RUNTIME_STORAGE_REFERENCE_CONFLICT
    );
    assert_eq!(
        worker.hosted.references.read(&cell).unwrap(),
        Value::Integer(1)
    );
}

#[test]
fn returning_or_capturing_authority_is_rejected_before_it_can_escape() {
    let mut worker = invocation(
        vec![
            abc(Opcode::ReserveReference, 1, 0, 0),
            abc(Opcode::MakeClosure, 2, 1, 1),
            return_unit(),
        ],
        vec![(vec![return_unit()], 1, Vec::new())],
        3,
    );
    let cell = storage(&mut worker, Value::Integer(1));
    worker.dispatch_one().unwrap();
    let error = worker
        .make_closure(AbcOperands {
            a: 2,
            b: 1,
            c: 1,
            auxiliary: 1,
        })
        .unwrap_err();
    assert_eq!(error.code, RUNTIME_STORAGE_REFERENCE_CONFLICT);
    let returned = worker.registers[1].clone();
    let Err(error) = worker.return_from_call(returned) else {
        panic!("reference returned");
    };
    assert_eq!(error.code, RUNTIME_STORAGE_REFERENCE_CONFLICT);
    worker.reference_scopes.clear();
    assert_eq!(
        worker.hosted.references.read(&cell).unwrap(),
        Value::Integer(1)
    );
}
