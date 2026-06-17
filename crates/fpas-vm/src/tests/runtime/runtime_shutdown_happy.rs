//! Successful [`Vm::run`] paths leave global shutdown set for pool teardown.
//!
//! **Documentation:** `docs/pascal/08-concurrency.md` (Phase 9), `docs/pascal/08-concurrency.md`,
//! `docs/pascal/std/task.md`

use crate::Vm;
use fpas_bytecode::{Chunk, Intrinsic, Op, TaskIntrinsic, Value};

use crate::tests::helpers::{build_zero_arg_function_chunk, emit_constant, loc};

// --- Positive: successful runs always shut down the pool ---

#[test]
fn successful_spawn_program_sets_shutdown_before_returning() {
    let callee = "Unit";
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
            chunk.emit(Op::Unit, loc());
            chunk.emit(Op::Return, loc());
        },
    );

    let mut vm = Vm::new(chunk);
    vm.run().expect("spawn + wait should succeed");
    assert!(
        vm.is_shutdown_for_tests(),
        "Vm::run must signal shutdown so pool workers can exit"
    );
}

#[test]
fn halt_only_program_without_pool_sets_shutdown() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Halt, loc());

    let mut vm = Vm::new(chunk);
    assert_eq!(vm.worker_pool_size_for_tests(), 0);
    vm.run().expect("halt-only");
    assert!(vm.is_shutdown_for_tests());
}

#[test]
fn repeated_runs_with_spawn_chunk_each_complete_and_set_shutdown() {
    let callee = "Nop";
    for _ in 0..24 {
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
                chunk.emit(Op::Halt, loc());
            },
            |chunk| {
                chunk.emit(Op::Unit, loc());
                chunk.emit(Op::Return, loc());
            },
        );

        let mut vm = Vm::new(chunk);
        vm.run().expect("repeated detached spawn runs");
        assert!(
            vm.is_shutdown_for_tests(),
            "each run must leave shutdown set for clean pool teardown"
        );
    }
}
