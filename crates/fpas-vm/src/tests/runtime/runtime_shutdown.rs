//! Runtime failures, global shutdown, and [`Vm::run`] error propagation.
//!
//! **Documentation:** `docs/future/parallel-vm.md` (Phase 9), `docs/pascal/08-concurrency.md`,
//! `docs/pascal/std/task.md`

use crate::Vm;
use fpas_bytecode::{Chunk, Intrinsic, Op, Value};
use fpas_diagnostics::codes::{
    RUNTIME_INVALID_TASK, RUNTIME_PROGRAM_PANIC, RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};

use crate::tests::helpers::{build_zero_arg_function_chunk, emit_constant, loc, run_err};

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
            chunk.emit(Op::Intrinsic(Intrinsic::TaskWait as u16), loc());
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

// --- Negative: pool failure after main halts ok (detached child) ---

#[test]
fn detached_child_panic_propagates_from_pool_when_main_halts_first() {
    let callee = "Boom";
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
            emit_constant(chunk, Value::Str("child".into()));
            chunk.emit(Op::Panic, loc());
        },
    );

    let err = run_err(chunk);
    assert_eq!(err.code, RUNTIME_PROGRAM_PANIC);
    assert!(
        err.message.contains("child"),
        "expected pool-reported panic: {}",
        err.message
    );
}

// --- Negative: main task failure (no background error to join) ---

#[test]
fn main_panic_returns_program_panic_without_hang_and_sets_shutdown() {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Str("oops".into()));
    chunk.emit(Op::Panic, loc());

    let mut vm = Vm::new(chunk);
    let err = vm.run().expect_err("main panic");
    assert_eq!(err.code, RUNTIME_PROGRAM_PANIC);
    assert!(vm.is_shutdown_for_tests());
}

#[test]
fn spawn_chunk_main_panic_still_sets_shutdown_with_worker_pool() {
    let callee = "NeverRuns";
    let mut chunk = Chunk::new();
    emit_constant(
        &mut chunk,
        Value::Function {
            name: callee.to_string(),
            captures: vec![],
        },
    );
    chunk.emit(Op::SpawnDetachedTask(0), loc());
    emit_constant(&mut chunk, Value::Str("main".into()));
    chunk.emit(Op::Panic, loc());

    let code_start = chunk.len();
    chunk
        .functions
        .insert(callee.to_ascii_lowercase(), (code_start, 0));
    chunk.emit(Op::Unit, loc());
    chunk.emit(Op::Return, loc());

    assert!(chunk.uses_spawn_tasks());
    let mut vm = Vm::new(chunk);
    assert!(vm.worker_pool_size_for_tests() > 0);

    let err = vm.run().expect_err("panic after detached spawn enqueue");
    assert_eq!(err.code, RUNTIME_PROGRAM_PANIC);
    assert!(vm.is_shutdown_for_tests());
}

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
