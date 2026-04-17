//! Execute path for [`Op::SpawnTask`] / [`Op::SpawnDetachedTask`]: callee resolution, arity checks,
//! [`SharedState::alloc_task_id`], [`crate::vm::SharedState::enqueue_task`], and stack effect
//! (retained [`Value::Task`] vs detached).
//!
//! **Documentation:** `docs/future/parallel-vm.md` (Phase 6), `docs/pascal/08-concurrency.md`

use fpas_bytecode::{Chunk, Intrinsic, Op, Value};
use fpas_diagnostics::codes::{
    INTERNAL_VM_INVARIANT_FAILURE, RUNTIME_INVALID_TASK, RUNTIME_VM_OPERAND_TYPE_MISMATCH,
    RUNTIME_WRONG_CALL_ARITY,
};

use crate::tests::helpers::{
    build_function_chunk, build_zero_arg_function_chunk, emit_constant, loc, run_err, run_ok_output,
};

// --- Positive: retained spawn, wait, observable result ---

#[test]
fn retained_spawn_wait_prints_child_return_value() {
    let callee = "ReturnFortyTwo";
    let chunk = build_zero_arg_function_chunk(
        callee,
        |chunk| {
            emit_constant(
                chunk,
                Value::Function {
                    name: callee.to_string(),
                    captures: vec![],
                },
            );
            chunk.emit(Op::SpawnTask(0), loc());
            chunk.emit(Op::Intrinsic(Intrinsic::TaskWait as u16), loc());
            chunk.emit(Op::PrintLn, loc());
        },
        |chunk| {
            emit_constant(chunk, Value::Integer(42));
            chunk.emit(Op::Return, loc());
        },
    );

    assert_eq!(run_ok_output(chunk), vec!["42"]);
}

#[test]
fn spawn_passes_two_arguments_and_child_returns_sum() {
    let callee = "AddTwo";
    let chunk = build_function_chunk(
        callee,
        2,
        |chunk| {
            emit_constant(chunk, Value::Integer(30));
            emit_constant(chunk, Value::Integer(12));
            emit_constant(
                chunk,
                Value::Function {
                    name: callee.to_string(),
                    captures: vec![],
                },
            );
            chunk.emit(Op::SpawnTask(2), loc());
            chunk.emit(Op::Intrinsic(Intrinsic::TaskWait as u16), loc());
            chunk.emit(Op::PrintLn, loc());
        },
        |chunk| {
            chunk.emit(Op::GetLocal(0), loc());
            chunk.emit(Op::GetLocal(1), loc());
            chunk.emit(Op::AddInt, loc());
            chunk.emit(Op::Return, loc());
        },
    );

    assert_eq!(run_ok_output(chunk), vec!["42"]);
}

#[test]
fn spawn_loads_closure_captures_onto_child_stack() {
    let callee = "ReturnCapture";
    let chunk = build_zero_arg_function_chunk(
        callee,
        |chunk| {
            emit_constant(
                chunk,
                Value::Function {
                    name: callee.to_string(),
                    captures: vec![Value::Integer(99)],
                },
            );
            chunk.emit(Op::SpawnTask(0), loc());
            chunk.emit(Op::Intrinsic(Intrinsic::TaskWait as u16), loc());
            chunk.emit(Op::PrintLn, loc());
        },
        |chunk| {
            chunk.emit(Op::GetLocal(0), loc());
            chunk.emit(Op::Return, loc());
        },
    );

    assert_eq!(run_ok_output(chunk), vec!["99"]);
}

#[test]
fn spawn_resolves_function_via_canonical_name_key() {
    // Chunk registers the callee under the lowercased key; the function *value* keeps mixed case.
    // Second lookup in spawn uses `canonical_name` of the value's name (see `exec_spawn_task`).
    let mut chunk = Chunk::new();
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "MixedCaseFn".to_string(),
            captures: vec![],
        },
    );
    chunk.emit(Op::SpawnTask(0), loc());
    chunk.emit(Op::Intrinsic(Intrinsic::TaskWait as u16), loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let code_start = chunk.len();
    chunk
        .functions
        .insert("mixedcasefn".to_string(), (code_start, 0));
    emit_constant(&mut chunk, Value::Integer(1));
    chunk.emit(Op::Return, loc());

    assert_eq!(run_ok_output(chunk), vec!["1"]);
}

// --- Positive: detached spawn leaves no task handle ---

#[test]
fn detached_spawn_does_not_leave_task_handle_on_stack() {
    let callee = "SideEffectOnly";
    let chunk = build_zero_arg_function_chunk(
        callee,
        |chunk| {
            emit_constant(
                chunk,
                Value::Function {
                    name: callee.to_string(),
                    captures: vec![],
                },
            );
            chunk.emit(Op::SpawnDetachedTask(0), loc());
            emit_constant(chunk, Value::Integer(77));
            chunk.emit(Op::PrintLn, loc());
        },
        |chunk| {
            chunk.emit(Op::Unit, loc());
            chunk.emit(Op::Return, loc());
        },
    );

    assert_eq!(run_ok_output(chunk), vec!["77"]);
}

// --- Negative: operand type, missing definition, arity ---

#[test]
fn spawn_task_rejects_non_function_callee() {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Integer(0));
    chunk.emit(Op::SpawnTask(0), loc());
    chunk.emit(Op::Halt, loc());

    let err = run_err(chunk);
    assert_eq!(err.code, RUNTIME_VM_OPERAND_TYPE_MISMATCH);
}

#[test]
fn spawn_task_rejects_missing_chunk_entry_for_function_name() {
    let mut chunk = Chunk::new();
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "NotRegistered".to_string(),
            captures: vec![],
        },
    );
    chunk.emit(Op::SpawnTask(0), loc());
    chunk.emit(Op::Halt, loc());

    let err = run_err(chunk);
    assert_eq!(err.code, RUNTIME_INVALID_TASK);
}

#[test]
fn spawn_detached_task_wrong_arity_reports_runtime_error() {
    let callee = "NeedTwoArgs";
    let chunk = build_function_chunk(
        callee,
        2,
        |chunk| {
            emit_constant(
                chunk,
                Value::Function {
                    name: callee.to_string(),
                    captures: vec![],
                },
            );
            chunk.emit(Op::SpawnDetachedTask(0), loc());
        },
        |chunk| {
            emit_constant(chunk, Value::Integer(0));
            emit_constant(chunk, Value::Integer(0));
            chunk.emit(Op::Return, loc());
        },
    );

    let err = run_err(chunk);
    assert_eq!(err.code, RUNTIME_WRONG_CALL_ARITY);
}

// --- Edge: stack layout / internal invariant ---

#[test]
fn spawn_task_with_missing_arguments_reports_internal_vm_error() {
    let callee = "ArityOne";
    let chunk = build_function_chunk(
        callee,
        1,
        |chunk| {
            emit_constant(
                chunk,
                Value::Function {
                    name: callee.to_string(),
                    captures: vec![],
                },
            );
            chunk.emit(Op::SpawnTask(1), loc());
        },
        |chunk| {
            emit_constant(chunk, Value::Integer(0));
            chunk.emit(Op::Return, loc());
        },
    );

    let err = run_err(chunk);
    assert_eq!(err.code, INTERNAL_VM_INVARIANT_FAILURE);
}
