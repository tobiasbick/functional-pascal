//! Panic paths: detached child after main halts, main panic with and without worker pool.
//!
//! **Documentation:** `docs/pascal/language/concurrency/README.md` (Phase 9), `docs/pascal/language/concurrency/README.md`,
//! `docs/pascal/std/concurrency/task.md`

use crate::Vm;
use fpas_bytecode::{Chunk, Op, Value};
use fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC;

use crate::tests::helpers::{build_zero_arg_function_chunk, emit_constant, loc, run_err};

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
    chunk.insert_function(callee.to_ascii_lowercase(), code_start, 0);
    chunk.emit(Op::Unit, loc());
    chunk.emit(Op::Return, loc());

    assert!(chunk.uses_spawn_tasks());
    let mut vm = Vm::new(chunk);
    assert!(vm.worker_pool_size_for_tests() > 0);

    let err = vm.run().expect_err("panic after detached spawn enqueue");
    assert_eq!(err.code, RUNTIME_PROGRAM_PANIC);
    assert!(vm.is_shutdown_for_tests());
}
