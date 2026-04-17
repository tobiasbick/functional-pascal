//! Cooperative scheduling: [`Op::Yield`], instruction timeslice, and main-task vs pool-task rules.
//!
//! **Documentation:** `docs/future/parallel-vm.md` (Phase 7), `docs/pascal/08-concurrency.md`

use crate::Vm;
use fpas_bytecode::{Chunk, Intrinsic, Op, Value};

use super::helpers::{build_zero_arg_function_chunk, emit_constant, loc, run_ok_output};

use crate::vm::Worker;
use std::sync::Arc;

use super::helpers::minimal_shared_state;

/// Each pair is two instructions (`Constant` + `Pop`); every instruction counts toward the
/// timeslice budget. Keep total cost comfortably above the VM `TIMESLICE` constant (see
/// `crates/fpas-vm/src/vm/mod.rs`, currently 256) when the test must force rescheduling.
fn emit_instruction_waste(chunk: &mut Chunk, instruction_pairs: usize) {
    for _ in 0..instruction_pairs {
        emit_constant(chunk, Value::Integer(0));
        chunk.emit(Op::Pop, loc());
    }
}

// --- Positive: explicit `Yield` on spawned task ---

#[test]
fn retained_spawn_child_yield_then_return_still_waits_correctly() {
    let callee = "YieldThenSeven";
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
            chunk.emit(Op::Yield, loc());
            emit_constant(chunk, Value::Integer(7));
            chunk.emit(Op::Return, loc());
        },
    );

    assert_eq!(run_ok_output(chunk), vec!["7"]);
}

#[test]
fn main_emits_many_yields_before_wait_child_still_completes() {
    let callee = "ReturnNine";
    let mut chunk = Chunk::new();
    emit_constant(
        &mut chunk,
        Value::Function {
            name: callee.to_string(),
            captures: vec![],
        },
    );
    chunk.emit(Op::SpawnTask(0), loc());
    for _ in 0..64 {
        chunk.emit(Op::Yield, loc());
    }
    chunk.emit(Op::Intrinsic(Intrinsic::TaskWait as u16), loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let code_start = chunk.len();
    chunk
        .functions
        .insert(callee.to_ascii_lowercase(), (code_start, 0));
    emit_constant(&mut chunk, Value::Integer(9));
    chunk.emit(Op::Return, loc());

    assert_eq!(run_ok_output(chunk), vec!["9"]);
}

#[test]
fn detached_spawn_child_yields_before_return() {
    let callee = "DetachedYield";
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
            emit_constant(chunk, Value::Integer(55));
            chunk.emit(Op::PrintLn, loc());
        },
        |chunk| {
            chunk.emit(Op::Yield, loc());
            chunk.emit(Op::Unit, loc());
            chunk.emit(Op::Return, loc());
        },
    );

    assert_eq!(run_ok_output(chunk), vec!["55"]);
}

// --- Positive: instruction timeslice lets two CPU-heavy spawned tasks finish ---

#[test]
fn two_wasteful_spawned_tasks_interleave_and_wait_all_completes() {
    // Each body runs far more than one timeslice of straight-line work so that, when only one
    // pool worker is available, the scheduler must suspend and re-queue each task repeatedly.
    // `WaitAll` proves both finished; two consecutive `Wait` ops would need extra stack handling.
    let slow_a = "SlowA";
    let slow_b = "SlowB";
    let mut chunk = Chunk::new();

    emit_constant(
        &mut chunk,
        Value::Function {
            name: slow_a.to_string(),
            captures: vec![],
        },
    );
    chunk.emit(Op::SpawnTask(0), loc());
    emit_constant(
        &mut chunk,
        Value::Function {
            name: slow_b.to_string(),
            captures: vec![],
        },
    );
    chunk.emit(Op::SpawnTask(0), loc());
    chunk.emit(Op::MakeArray(2), loc());
    chunk.emit(Op::Intrinsic(Intrinsic::TaskWaitAll as u16), loc());
    emit_constant(&mut chunk, Value::Str("both_done".to_string()));
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let fn_a_start = chunk.len();
    chunk
        .functions
        .insert(slow_a.to_ascii_lowercase(), (fn_a_start, 0));
    emit_instruction_waste(&mut chunk, 400);
    emit_constant(&mut chunk, Value::Integer(10));
    chunk.emit(Op::Return, loc());

    let fn_b_start = chunk.len();
    chunk
        .functions
        .insert(slow_b.to_ascii_lowercase(), (fn_b_start, 0));
    emit_instruction_waste(&mut chunk, 400);
    emit_constant(&mut chunk, Value::Integer(32));
    chunk.emit(Op::Return, loc());

    assert_eq!(run_ok_output(chunk), vec!["both_done"]);
}

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
            chunk.emit(Op::Intrinsic(Intrinsic::TaskWait as u16), loc());
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

// --- Edge / unit: main task never moves to the shared ready queue ---

#[test]
fn main_task_timeslice_does_not_enqueue_task_zero() {
    let mut c = Chunk::new();
    emit_instruction_waste(&mut c, 200);
    c.emit(Op::Halt, loc());
    let shared = Arc::new(minimal_shared_state(c));
    let mut main = Worker::new_main(Arc::clone(&shared));
    main.run().expect("main should finish");
    assert_eq!(main.current_task_id, 0);
    assert!(
        shared
            .task_queue
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .all(|t| t.id != 0),
        "task id 0 must never appear on the shared ready queue"
    );
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
            chunk.emit(Op::Intrinsic(Intrinsic::TaskWait as u16), loc());
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
