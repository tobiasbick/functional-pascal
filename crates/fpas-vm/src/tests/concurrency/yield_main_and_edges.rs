//! Main-only `Yield`, single-child reschedule, and spawn+`Yield` without runtime error.
//!
//! **Documentation:** `docs/pascal/08-concurrency.md` (Phase 7), `docs/pascal/08-concurrency.md`

use crate::Vm;
use fpas_bytecode::{Chunk, Intrinsic, Op, TaskIntrinsic, Value};

use crate::tests::helpers::{build_zero_arg_function_chunk, emit_constant, loc, run_ok_output};

// --- Edge: `Yield` alone does not imply a worker pool ---

#[test]
fn yield_only_chunk_without_spawn_runs_without_worker_pool() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Yield, loc());
    chunk.emit(Op::Yield, loc());
    chunk.emit(Op::Unit, loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let mut vm = Vm::new(chunk);
    assert_eq!(vm.worker_pool_size_for_tests(), 0);
    vm.run().expect("yield-only main program should succeed");
    assert_eq!(vm.output().lines, vec!["()"]);
}

// --- Edge: spawned task yields when no other work is queued (no-op reschedule) ---

#[test]
fn single_spawned_task_yield_only_child_still_returns() {
    let callee = "OnlyYield";
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
            chunk.emit(
                Op::Intrinsic(u16::from(Intrinsic::Task(TaskIntrinsic::Wait))),
                loc(),
            );
            chunk.emit(Op::PrintLn, loc());
        },
        |chunk| {
            chunk.emit(Op::Yield, loc());
            emit_constant(chunk, Value::Integer(1));
            chunk.emit(Op::Return, loc());
        },
    );

    assert_eq!(run_ok_output(chunk), vec!["1"]);
}

// --- Negative: `Yield` is not an error (still exercise error-free path with spawn) ---

#[test]
fn spawn_with_yield_opcode_does_not_fail_runtime() {
    let callee = "JustYield";
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
            chunk.emit(
                Op::Intrinsic(u16::from(Intrinsic::Task(TaskIntrinsic::Wait))),
                loc(),
            );
        },
        |chunk| {
            chunk.emit(Op::Yield, loc());
            chunk.emit(Op::Unit, loc());
            chunk.emit(Op::Return, loc());
        },
    );

    let mut vm = Vm::new(chunk);
    vm.run().expect("Yield in spawned task must not error");
}
