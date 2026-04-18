//! Spawn operand validation, missing definitions, arity, internal invariants.
//!
//! **Documentation:** `docs/rust/parallel-vm.md` (Phase 6), `docs/pascal/08-concurrency.md`

use fpas_bytecode::{Chunk, Op, Value};
use fpas_diagnostics::codes::{
    INTERNAL_VM_INVARIANT_FAILURE, RUNTIME_INVALID_TASK, RUNTIME_VM_OPERAND_TYPE_MISMATCH,
    RUNTIME_WRONG_CALL_ARITY,
};

use crate::tests::helpers::{build_function_chunk, emit_constant, loc, run_err};

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
