//! Task-wait validation and empty `WaitAll` after spawn under shutdown semantics.
//!
//! **Documentation:** `docs/future/parallel-vm.md` (Phase 9), `docs/pascal/08-concurrency.md`,
//! `docs/pascal/std/task.md`

use crate::Vm;
use fpas_bytecode::{Chunk, Intrinsic, Op, Value};
use fpas_diagnostics::codes::{RUNTIME_INVALID_TASK, RUNTIME_VM_OPERAND_TYPE_MISMATCH};

use crate::tests::helpers::{build_zero_arg_function_chunk, emit_constant, loc, run_err};

// --- Negative: waiter path (retained spawn) ---

#[test]
fn wait_twice_on_same_task_second_wait_is_invalid_task() {
    let callee = "N";
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
            chunk.emit(Op::Dup, loc());
            chunk.emit(Op::Intrinsic(Intrinsic::TaskWait as u16), loc());
            chunk.emit(Op::Pop, loc());
            chunk.emit(Op::Intrinsic(Intrinsic::TaskWait as u16), loc());
        },
        |chunk| {
            chunk.emit(Op::Unit, loc());
            chunk.emit(Op::Return, loc());
        },
    );

    let err = run_err(chunk);
    assert_eq!(err.code, RUNTIME_INVALID_TASK);
}

#[test]
fn wait_non_task_operand_errors_without_shutdown_semantics() {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Integer(0));
    chunk.emit(Op::Intrinsic(Intrinsic::TaskWait as u16), loc());
    chunk.emit(Op::Halt, loc());

    let mut vm = Vm::new(chunk);
    let err = vm.run().expect_err("Wait type mismatch");
    assert_eq!(err.code, RUNTIME_VM_OPERAND_TYPE_MISMATCH);
    assert!(
        vm.is_shutdown_for_tests(),
        "run must still complete teardown after main failure"
    );
}

// --- Edge: empty WaitAll + error paths ---

#[test]
fn wait_all_empty_succeeds_and_sets_shutdown_with_spawn_pool() {
    let callee = "Side";
    let mut chunk = Chunk::new();
    emit_constant(
        &mut chunk,
        Value::Function {
            name: callee.to_string(),
            captures: vec![],
        },
    );
    chunk.emit(Op::SpawnTask(0), loc());
    chunk.emit(Op::Intrinsic(Intrinsic::TaskWait as u16), loc());
    chunk.emit(Op::MakeArray(0), loc());
    chunk.emit(Op::Intrinsic(Intrinsic::TaskWaitAll as u16), loc());
    chunk.emit(Op::Halt, loc());

    let code_start = chunk.len();
    chunk
        .functions
        .insert(callee.to_ascii_lowercase(), (code_start, 0));
    chunk.emit(Op::Unit, loc());
    chunk.emit(Op::Return, loc());

    let mut vm = Vm::new(chunk);
    vm.run().expect("WaitAll([]) with spawn bytecode");
    assert!(vm.is_shutdown_for_tests());
}
