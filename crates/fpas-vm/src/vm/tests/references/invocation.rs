//! Direct, indirect, forwarded and unwound synchronous references.

use super::{fixture::*, *};

#[test]
fn direct_return_releases_authority_even_with_retained_register_copies() {
    let mut worker = invocation(
        vec![
            abc(Opcode::ReserveReference, 1, 0, 0),
            abc_aux(Opcode::CallDirect, NO_REGISTER, 1, 1, 1),
            abc(Opcode::CellRead, 2, 0, 0),
            return_unit(),
        ],
        vec![(set_nine(), 2, vec![0])],
        3,
    );
    let cell = storage(&mut worker, Value::Integer(1));
    run(&mut worker).unwrap();
    assert_eq!(
        worker.hosted.references.read(&cell).unwrap(),
        Value::Integer(9)
    );
    assert_eq!(worker.registers[2], Value::Integer(9));
    let Value::Reference(retained) = &worker.registers[1] else {
        panic!("retained copy");
    };
    assert!(matches!(
        retained.read(),
        Err(fpas_bytecode::ReferencePathError::Access(
            fpas_bytecode::ReferenceError::Released
        ))
    ));
}

#[test]
fn indirect_tail_call_preserves_overlapping_argument_window_and_cleanup() {
    let mut worker = invocation(
        vec![
            abc(Opcode::MakeClosure, 1, 1, 0),
            abc(Opcode::ReserveReference, 2, 0, 0),
            abc_aux(Opcode::TailCallValue, NO_REGISTER, 1, 2, 1),
            return_unit(),
        ],
        vec![(set_nine(), 2, vec![0])],
        3,
    );
    let cell = storage(&mut worker, Value::Integer(1));
    run(&mut worker).unwrap();
    assert_eq!(
        worker.hosted.references.read(&cell).unwrap(),
        Value::Integer(9)
    );
}

#[test]
fn direct_tail_call_releases_both_frames() {
    let mut worker = invocation(
        vec![
            abc(Opcode::ReserveReference, 1, 0, 0),
            abc_aux(Opcode::TailCall, NO_REGISTER, 1, 1, 1),
            return_unit(),
        ],
        vec![(set_nine(), 2, vec![0])],
        2,
    );
    let cell = storage(&mut worker, Value::Integer(1));
    run(&mut worker).unwrap();
    assert_eq!(
        worker.hosted.references.read(&cell).unwrap(),
        Value::Integer(9)
    );
}

#[test]
fn forwarding_restores_parent_reads_and_writes() {
    let parent = vec![
        abc(Opcode::ReserveReference, 2, 0, 0),
        abc(Opcode::ReadReference, 1, 0, 0),
        abc_aux(Opcode::CallDirect, NO_REGISTER, 2, 2, 1),
        abc(Opcode::ReadReference, 1, 0, 0),
        abc(Opcode::WriteReference, 0, 1, 0),
        return_unit(),
    ];
    let mut worker = invocation(
        vec![
            abc(Opcode::ReserveReference, 1, 0, 0),
            abc_aux(Opcode::CallDirect, NO_REGISTER, 1, 1, 1),
            return_unit(),
        ],
        vec![(parent, 3, vec![0]), (set_nine(), 2, vec![0])],
        2,
    );
    let cell = storage(&mut worker, Value::Integer(1));
    run(&mut worker).unwrap();
    assert_eq!(
        worker.hosted.references.read(&cell).unwrap(),
        Value::Integer(9)
    );
}

#[test]
fn panic_after_write_releases_all_authority_without_rolling_back_effects() {
    let mut callee = set_nine();
    callee.insert(2, abc(Opcode::Panic, 1, 0, 0));
    let mut worker = invocation(
        vec![
            abc(Opcode::ReserveReference, 1, 0, 0),
            abc_aux(Opcode::CallDirect, NO_REGISTER, 1, 1, 1),
            return_unit(),
        ],
        vec![(callee, 2, vec![0])],
        2,
    );
    let cell = storage(&mut worker, Value::Integer(1));
    assert_eq!(
        run(&mut worker).unwrap_err().code,
        fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC
    );
    assert_eq!(
        worker.hosted.references.read(&cell).unwrap(),
        Value::Integer(9)
    );
    worker.hosted.references.reserve(&cell).unwrap();
}

#[test]
fn two_register_copies_cannot_supply_two_var_parameters_from_one_root() {
    let mut worker = invocation(
        vec![
            abc(Opcode::ReserveReference, 1, 0, 0),
            abc(Opcode::Move, 2, 1, 0),
            abc_aux(Opcode::CallDirect, NO_REGISTER, 1, 1, 2),
            return_unit(),
        ],
        vec![(vec![return_unit()], 2, vec![0, 1])],
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

#[test]
fn callback_cache_releases_each_successful_invocation() {
    let mut worker = invocation(vec![return_unit()], vec![(set_nine(), 2, vec![0])], 1);
    let cell = storage(&mut worker, Value::Integer(1));
    let callback = Value::function(
        fpas_bytecode::FunctionId::new(1),
        "set".to_string(),
        Vec::new(),
    );
    for _ in 0..2 {
        let reference = Arc::new(fpas_bytecode::SelectedReference::root(
            worker.hosted.references.reserve(&cell).unwrap(),
        ));
        worker
            .call_callback_sync(&callback, [Value::Reference(Arc::clone(&reference))])
            .unwrap();
        assert_eq!(
            worker.hosted.references.read(&cell).unwrap(),
            Value::Integer(9)
        );
        assert!(reference.read().is_err());
    }
}
