//! Saved workers, cached callbacks and debugger exits own their leases explicitly.

use super::{fixture::*, *};

fn paused_invocation() -> (Worker, Arc<Mutex<Value>>) {
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
    worker.dispatch_one().unwrap();
    worker.dispatch_one().unwrap();
    (worker, cell)
}

#[test]
fn suspended_invocation_keeps_authority_until_resumed_worker_returns() {
    let (mut worker, cell) = paused_invocation();
    let saved = worker.take_task_state();
    assert!(worker.hosted.references.read(&cell).is_err());
    let mut resumed = worker.worker_for_task(saved);
    run(&mut resumed).unwrap();
    assert_eq!(
        worker.hosted.references.read(&cell).unwrap(),
        Value::Integer(9)
    );
}

#[test]
fn dropping_suspended_state_releases_even_retained_parameter_copies() {
    let (mut worker, cell) = paused_invocation();
    let Value::Reference(retained) = worker.registers[worker.base].clone() else {
        panic!("active reference");
    };
    drop(worker.take_task_state());
    assert_eq!(
        worker.hosted.references.read(&cell).unwrap(),
        Value::Integer(1)
    );
    assert!(retained.read().is_err());
}

#[test]
fn forced_frame_unwind_releases_child_and_preserves_current_parent() {
    let (mut worker, cell) = paused_invocation();
    worker.reference_scopes.truncate(1);
    assert_eq!(
        worker.hosted.references.read(&cell).unwrap(),
        Value::Integer(1)
    );
    worker.hosted.references.reserve(&cell).unwrap();
}

#[test]
fn cached_callback_failure_releases_partial_writes_before_next_invocation() {
    let mut failing = set_nine();
    failing.insert(2, abc(Opcode::Panic, 1, 0, 0));
    let mut worker = invocation(
        vec![return_unit()],
        vec![(failing, 2, vec![0]), (set_nine(), 2, vec![0])],
        1,
    );
    let cell = storage(&mut worker, Value::Integer(1));
    for (target, fails) in [(1, true), (2, false)] {
        let callback = Value::function(
            fpas_bytecode::FunctionId::new(target),
            "callback".to_string(),
            Vec::new(),
        );
        let reference = Arc::new(fpas_bytecode::SelectedReference::root(
            worker.hosted.references.reserve(&cell).unwrap(),
        ));
        let result =
            worker.call_callback_sync(&callback, [Value::Reference(Arc::clone(&reference))]);
        assert_eq!(result.is_err(), fails);
        assert_eq!(
            worker.hosted.references.read(&cell).unwrap(),
            Value::Integer(9)
        );
        assert!(reference.read().is_err());
    }
}

#[test]
fn two_distinct_storage_roots_can_be_mutated_in_one_invocation() {
    let callee = vec![
        abx(Opcode::LoadConstant, 2, 0),
        abc(Opcode::WriteReference, 0, 2, 0),
        abc(Opcode::WriteReference, 1, 2, 0),
        return_unit(),
    ];
    let mut worker = invocation(
        vec![
            abc(Opcode::ReserveReference, 2, 0, 0),
            abc(Opcode::ReserveReference, 3, 1, 0),
            abc_aux(Opcode::CallDirect, NO_REGISTER, 1, 2, 2),
            return_unit(),
        ],
        vec![(callee, 3, vec![0, 1])],
        4,
    );
    let first = storage(&mut worker, Value::Integer(1));
    let second = Arc::new(Mutex::new(Value::Integer(2)));
    worker
        .store_register(1, Value::Cell(Arc::clone(&second)))
        .unwrap();
    run(&mut worker).unwrap();
    assert_eq!(
        worker.hosted.references.read(&first).unwrap(),
        Value::Integer(9)
    );
    assert_eq!(
        worker.hosted.references.read(&second).unwrap(),
        Value::Integer(9)
    );
}

#[test]
fn repeated_explicit_release_discards_completed_leases_from_the_frame() {
    let mut worker = invocation(vec![return_unit()], Vec::new(), 2);
    let cell = storage(&mut worker, Value::Integer(1));
    for _ in 0..256 {
        worker.reserve_reference(operands(1, 0)).unwrap();
        worker.release_reference(operands(1, 0)).unwrap();
        assert!(!worker.reference_scopes.current_has_leases());
    }
    assert_eq!(
        worker.hosted.references.read(&cell).unwrap(),
        Value::Integer(1)
    );
}
