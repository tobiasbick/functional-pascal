//! Spawn completion failures, arity, and invalid double-`Wait` on the same task handle.
//!
//! **Documentation:** `docs/pascal/std/concurrency/task.md`, `docs/pascal/language/concurrency/README.md`

use fpas_bytecode::{Chunk, Intrinsic, Op, TaskIntrinsic, Value};
use fpas_diagnostics::Diagnostic;
use fpas_diagnostics::codes::{
    INTERNAL_VM_INVARIANT_FAILURE, RUNTIME_INVALID_TASK, RUNTIME_WRONG_CALL_ARITY,
};
use std::sync::mpsc;
use std::time::Duration;

use crate::Vm;
use crate::tests::helpers::{
    build_function_chunk, build_zero_arg_function_chunk, emit_constant, loc, run_err,
};

fn run_with_timeout(chunk: Chunk) -> Diagnostic {
    let (sender, receiver) = mpsc::sync_channel(1);
    let handle = std::thread::spawn(move || {
        let mut vm = Vm::new(chunk);
        sender
            .send(vm.run())
            .expect("test receiver should remain connected");
    });

    let result = receiver
        .recv_timeout(Duration::from_secs(2))
        .expect("retained task failure must not leave Wait blocked");
    handle.join().expect("VM thread should not panic");
    result.expect_err("malformed spawned task should fail")
}

#[test]
fn spawn_task_with_wrong_arity_reports_runtime_error() {
    let function_name = "NeedOneArg";
    let chunk = build_function_chunk(
        function_name,
        1,
        |chunk| {
            emit_constant(
                chunk,
                Value::function(function_name.to_string(), vec![], false),
            );
            chunk.emit(Op::SpawnTask(0), loc());
        },
        |chunk| {
            emit_constant(chunk, Value::Integer(0));
            chunk.emit(Op::Return, loc());
        },
    );

    let err = run_err(chunk);
    assert_eq!(err.code, RUNTIME_WRONG_CALL_ARITY);
}

#[test]
fn waiting_twice_on_same_task_reports_runtime_error() {
    let function_name = "ReturnSeven";
    let chunk = build_zero_arg_function_chunk(
        function_name,
        |chunk| {
            emit_constant(
                chunk,
                Value::function(function_name.to_string(), vec![], false),
            );
            chunk.emit(Op::SpawnTask(0), loc());
            chunk.emit(Op::Dup, loc());
            chunk.emit(
                Op::Intrinsic(u16::from(Intrinsic::Task(TaskIntrinsic::Wait))),
                loc(),
            );
            chunk.emit(Op::Pop, loc());
            chunk.emit(
                Op::Intrinsic(u16::from(Intrinsic::Task(TaskIntrinsic::Wait))),
                loc(),
            );
        },
        |chunk| {
            emit_constant(chunk, Value::Integer(7));
            chunk.emit(Op::Return, loc());
        },
    );

    let err = run_err(chunk);
    assert_eq!(err.code, RUNTIME_INVALID_TASK);
}

#[test]
fn retained_spawn_reaching_halt_completes_as_failure_without_hanging_wait() {
    let function_name = "Halts";
    let chunk = build_zero_arg_function_chunk(
        function_name,
        |chunk| {
            emit_constant(
                chunk,
                Value::function(function_name.to_string(), vec![], false),
            );
            chunk.emit(Op::SpawnTask(0), loc());
            chunk.emit(
                Op::Intrinsic(u16::from(Intrinsic::Task(TaskIntrinsic::Wait))),
                loc(),
            );
        },
        |chunk| {
            chunk.emit(Op::Halt, loc());
        },
    );

    let error = run_with_timeout(chunk);
    assert_eq!(error.code, INTERNAL_VM_INVARIANT_FAILURE);
    assert!(
        error
            .message
            .contains("Halt is invalid during spawned task")
    );
}

#[test]
fn retained_spawn_reaching_code_boundary_fails_without_using_argument_as_result() {
    let function_name = "FallsOff";
    let chunk = build_function_chunk(
        function_name,
        1,
        |chunk| {
            emit_constant(chunk, Value::Integer(41));
            emit_constant(
                chunk,
                Value::function(function_name.to_string(), vec![], false),
            );
            chunk.emit(Op::SpawnTask(1), loc());
            chunk.emit(
                Op::Intrinsic(u16::from(Intrinsic::Task(TaskIntrinsic::Wait))),
                loc(),
            );
        },
        |chunk| {
            chunk.emit(Op::Unit, loc());
        },
    );

    let error = run_with_timeout(chunk);
    assert_eq!(error.code, INTERNAL_VM_INVARIANT_FAILURE);
    assert!(error.message.contains("code boundary"));
    assert!(error.message.contains("context=spawned task"));
}

#[test]
fn spawn_target_at_code_length_is_rejected_before_task_registration() {
    let function_name = "MissingBody";
    let mut chunk = Chunk::new();
    emit_constant(
        &mut chunk,
        Value::function(function_name.to_string(), vec![], false),
    );
    chunk.emit(Op::SpawnTask(0), loc());
    chunk.emit(Op::Halt, loc());
    chunk.insert_function(function_name.to_string(), chunk.len(), 0);

    let error = run_err(chunk);
    assert_eq!(error.code, INTERNAL_VM_INVARIANT_FAILURE);
    assert!(error.message.contains("Bytecode entry"));
}
