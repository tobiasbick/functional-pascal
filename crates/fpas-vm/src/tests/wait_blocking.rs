//! Blocking [`Std.Task.Wait`] / [`Std.Task.WaitAll`]: condvar wait (no hot-spin), main vs pool progress.
//!
//! **Documentation:** `docs/future/parallel-vm.md` (Phase 8), `docs/pascal/std/task.md`, `docs/pascal/08-concurrency.md`

use crate::Vm;
use fpas_bytecode::{Chunk, Intrinsic, Op, Value};
use fpas_diagnostics::codes::{RUNTIME_VM_OPERAND_TYPE_MISMATCH, RUNTIME_VM_SHUTDOWN};

use super::helpers::{build_zero_arg_function_chunk, emit_constant, loc, run_err, run_ok_output};

/// Same pattern as `yield_scheduling`: force rescheduling on spawned tasks (see `TIMESLICE` in VM).
fn emit_instruction_waste(chunk: &mut Chunk, instruction_pairs: usize) {
    for _ in 0..instruction_pairs {
        emit_constant(chunk, Value::Integer(0));
        chunk.emit(Op::Pop, loc());
    }
}

// --- Positive: Wait completes under load; child runs on pool ---

#[test]
fn wait_succeeds_when_child_runs_long_enough_to_need_timeslice() {
    let callee = "BusyThenThree";
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
            emit_instruction_waste(chunk, 200);
            emit_constant(chunk, Value::Integer(3));
            chunk.emit(Op::Return, loc());
        },
    );

    assert_eq!(run_ok_output(chunk), vec!["3"]);
}

#[test]
fn wait_then_second_spawn_and_wait_is_independent() {
    let f = "Inc";
    let chunk = build_zero_arg_function_chunk(
        f,
        |chunk| {
            for _ in 0..2 {
                emit_constant(
                    chunk,
                    Value::Function {
                        name: f.to_string(),
                        captures: vec![],
                    },
                );
                chunk.emit(Op::SpawnTask(0), loc());
                chunk.emit(Op::Intrinsic(Intrinsic::TaskWait as u16), loc());
            }
            chunk.emit(Op::AddInt, loc());
            chunk.emit(Op::PrintLn, loc());
        },
        |chunk| {
            emit_constant(chunk, Value::Integer(10));
            chunk.emit(Op::Return, loc());
        },
    );

    assert_eq!(run_ok_output(chunk), vec!["20"]);
}

// --- Positive: WaitAll ---

#[test]
fn wait_all_empty_array_completes_immediately() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::MakeArray(0), loc());
    chunk.emit(Op::Intrinsic(Intrinsic::TaskWaitAll as u16), loc());
    chunk.emit(Op::Halt, loc());

    let mut vm = Vm::new(chunk);
    vm.run().expect("empty WaitAll should succeed");
}

#[test]
fn wait_all_two_tasks_barrier_then_wait_each_prints() {
    let callee = "Seven";
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
            emit_constant(
                chunk,
                Value::Function {
                    name: callee.to_string(),
                    captures: vec![],
                },
            );
            chunk.emit(Op::SpawnTask(0), loc());
            chunk.emit(Op::Dup, loc());
            chunk.emit(Op::Dup, loc());
            chunk.emit(Op::MakeArray(2), loc());
            chunk.emit(Op::Intrinsic(Intrinsic::TaskWaitAll as u16), loc());
            chunk.emit(Op::Intrinsic(Intrinsic::TaskWait as u16), loc());
            chunk.emit(Op::PrintLn, loc());
            chunk.emit(Op::Intrinsic(Intrinsic::TaskWait as u16), loc());
            chunk.emit(Op::PrintLn, loc());
        },
        |chunk| {
            emit_constant(chunk, Value::Integer(7));
            chunk.emit(Op::Return, loc());
        },
    );

    assert_eq!(run_ok_output(chunk), vec!["7", "7"]);
}

#[test]
fn wait_all_three_tasks_busy_children_then_barrier() {
    let callee = "N";
    let chunk = build_zero_arg_function_chunk(
        callee,
        |chunk| {
            for _ in 0..3 {
                emit_constant(
                    chunk,
                    Value::Function {
                        name: callee.to_string(),
                        captures: vec![],
                    },
                );
                chunk.emit(Op::SpawnTask(0), loc());
            }
            chunk.emit(Op::MakeArray(3), loc());
            chunk.emit(Op::Intrinsic(Intrinsic::TaskWaitAll as u16), loc());
            chunk.emit(Op::Halt, loc());
        },
        |chunk| {
            emit_instruction_waste(chunk, 120);
            chunk.emit(Op::Unit, loc());
            chunk.emit(Op::Return, loc());
        },
    );

    let mut vm = Vm::new(chunk);
    vm.run().expect("WaitAll with three busy tasks");
}

// --- Negative ---

#[test]
fn wait_all_rejects_integer_element() {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Integer(1));
    emit_constant(&mut chunk, Value::Integer(2));
    chunk.emit(Op::MakeArray(2), loc());
    chunk.emit(Op::Intrinsic(Intrinsic::TaskWaitAll as u16), loc());
    chunk.emit(Op::Halt, loc());

    let err = run_err(chunk);
    assert_eq!(err.code, RUNTIME_VM_OPERAND_TYPE_MISMATCH);
}

#[test]
fn wait_on_child_that_panics_surfaces_shutdown_to_waiter() {
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
            chunk.emit(Op::SpawnTask(0), loc());
            chunk.emit(Op::Intrinsic(Intrinsic::TaskWait as u16), loc());
        },
        |chunk| {
            emit_constant(chunk, Value::Str("x".into()));
            chunk.emit(Op::Panic, loc());
        },
    );

    let err = run_err(chunk);
    assert_eq!(err.code, RUNTIME_VM_SHUTDOWN);
}

#[test]
fn wait_all_on_array_when_child_panicked_reports_shutdown() {
    let ok = "OkTask";
    let bad = "BadTask";
    let mut chunk = Chunk::new();

    emit_constant(
        &mut chunk,
        Value::Function {
            name: ok.to_string(),
            captures: vec![],
        },
    );
    chunk.emit(Op::SpawnTask(0), loc());

    emit_constant(
        &mut chunk,
        Value::Function {
            name: bad.to_string(),
            captures: vec![],
        },
    );
    chunk.emit(Op::SpawnTask(0), loc());

    chunk.emit(Op::Dup, loc());
    chunk.emit(Op::Dup, loc());
    chunk.emit(Op::MakeArray(2), loc());
    chunk.emit(Op::Intrinsic(Intrinsic::TaskWaitAll as u16), loc());
    chunk.emit(Op::Halt, loc());

    let code_start = chunk.len();
    chunk
        .functions
        .insert(ok.to_ascii_lowercase(), (code_start, 0));
    emit_constant(&mut chunk, Value::Integer(1));
    chunk.emit(Op::Return, loc());

    let code_start_bad = chunk.len();
    chunk
        .functions
        .insert(bad.to_ascii_lowercase(), (code_start_bad, 0));
    emit_constant(&mut chunk, Value::Str("e".into()));
    chunk.emit(Op::Panic, loc());

    let err = run_err(chunk);
    assert_eq!(err.code, RUNTIME_VM_SHUTDOWN);
}

// --- Edge: main yields while waiting (stack / IP consistency) ---

#[test]
fn main_yields_between_spawns_wait_all_still_completes() {
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
            chunk.emit(Op::Yield, loc());
            emit_constant(
                chunk,
                Value::Function {
                    name: callee.to_string(),
                    captures: vec![],
                },
            );
            chunk.emit(Op::SpawnTask(0), loc());
            chunk.emit(Op::Dup, loc());
            chunk.emit(Op::Dup, loc());
            chunk.emit(Op::MakeArray(2), loc());
            chunk.emit(Op::Intrinsic(Intrinsic::TaskWaitAll as u16), loc());
            chunk.emit(Op::Halt, loc());
        },
        |chunk| {
            chunk.emit(Op::Unit, loc());
            chunk.emit(Op::Return, loc());
        },
    );

    let mut vm = Vm::new(chunk);
    vm.run().expect("WaitAll after main yields");
}
