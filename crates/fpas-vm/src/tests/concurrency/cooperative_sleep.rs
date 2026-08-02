//! Cooperative `Std.Time.Sleep` scheduling for spawned tasks.
//!
//! **Documentation:** `docs/pascal/std/host/time.md`,
//! `docs/pascal/language/concurrency/scheduling.md`.

use crate::Vm;
use crate::tests::helpers::{build_zero_arg_function_chunk, emit_constant, loc};
use fpas_bytecode::{Intrinsic, Op, TaskIntrinsic, TimeIntrinsic, Value};
use fpas_diagnostics::codes::RUNTIME_NUMERIC_DOMAIN_ERROR;
use std::time::{Duration, Instant};

fn sleep_wait_all_chunk(task_count: u16, milliseconds: i64) -> fpas_bytecode::Chunk {
    let callee = "Sleeper";
    build_zero_arg_function_chunk(
        callee,
        |chunk| {
            for _ in 0..task_count {
                emit_constant(
                    chunk,
                    Value::function(callee.to_string(), Vec::new(), false),
                );
                chunk.emit(Op::SpawnTask(0), loc());
            }
            chunk.emit(Op::MakeArray(task_count), loc());
            chunk.emit(
                Op::Intrinsic(u16::from(Intrinsic::Task(TaskIntrinsic::WaitAll))),
                loc(),
            );
            chunk.emit(Op::Unit, loc());
            chunk.emit(Op::Halt, loc());
        },
        |chunk| {
            emit_constant(chunk, Value::Integer(milliseconds));
            chunk.emit(
                Op::Intrinsic(u16::from(Intrinsic::Time(TimeIntrinsic::Sleep))),
                loc(),
            );
            chunk.emit(Op::Unit, loc());
            chunk.emit(Op::Return, loc());
        },
    )
}

#[test]
fn spawned_sleeps_release_the_only_pool_worker() {
    let mut vm = Vm::new(sleep_wait_all_chunk(12, 40));
    vm.set_worker_pool_size_for_tests(1);

    let started = Instant::now();
    vm.run().expect("cooperative sleeps should finish");

    assert!(
        started.elapsed() < Duration::from_millis(350),
        "twelve 40 ms sleeps should overlap on one pool worker"
    );
}

#[test]
fn spawned_sleep_rejects_negative_duration() {
    let mut vm = Vm::new(sleep_wait_all_chunk(1, -1));
    vm.set_worker_pool_size_for_tests(1);

    let error = vm.run().expect_err("negative spawned sleep should fail");

    assert_eq!(error.code, RUNTIME_NUMERIC_DOMAIN_ERROR);
}

#[test]
fn main_task_sleep_remains_blocking() {
    let mut chunk = fpas_bytecode::Chunk::new();
    emit_constant(&mut chunk, Value::Integer(20));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Time(TimeIntrinsic::Sleep))),
        loc(),
    );
    chunk.emit(Op::Unit, loc());
    chunk.emit(Op::Halt, loc());

    let started = Instant::now();
    Vm::new(chunk).run().expect("main sleep should finish");

    assert!(started.elapsed() >= Duration::from_millis(15));
}

#[test]
fn sleeping_child_wakes_parent_waiting_on_the_only_pool_worker() {
    let parent = "Parent";
    let child = "Child";
    let mut chunk = fpas_bytecode::Chunk::new();

    emit_constant(
        &mut chunk,
        Value::function(parent.to_string(), Vec::new(), false),
    );
    chunk.emit(Op::SpawnTask(0), loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Task(TaskIntrinsic::Wait))),
        loc(),
    );
    chunk.emit(Op::Halt, loc());

    let parent_start = chunk.len();
    chunk.insert_function(parent.to_ascii_lowercase(), parent_start, 0);
    emit_constant(
        &mut chunk,
        Value::function(child.to_string(), Vec::new(), false),
    );
    chunk.emit(Op::SpawnTask(0), loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Task(TaskIntrinsic::Wait))),
        loc(),
    );
    chunk.emit(Op::Return, loc());

    let child_start = chunk.len();
    chunk.insert_function(child.to_ascii_lowercase(), child_start, 0);
    emit_constant(&mut chunk, Value::Integer(20));
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Time(TimeIntrinsic::Sleep))),
        loc(),
    );
    chunk.emit(Op::Unit, loc());
    chunk.emit(Op::Return, loc());

    let mut vm = Vm::new(chunk);
    vm.set_worker_pool_size_for_tests(1);
    vm.run()
        .expect("timer-ready child should release the parent's result wait");
}

#[test]
fn main_completion_explicitly_cancels_detached_sleepers() {
    let callee = "Sleeper";
    let chunk = build_zero_arg_function_chunk(
        callee,
        |chunk| {
            emit_constant(
                chunk,
                Value::function(callee.to_string(), Vec::new(), false),
            );
            chunk.emit(Op::SpawnDetachedTask(0), loc());
            emit_constant(chunk, Value::Integer(250));
            chunk.emit(
                Op::Intrinsic(u16::from(Intrinsic::Time(TimeIntrinsic::Sleep))),
                loc(),
            );
            chunk.emit(Op::Halt, loc());
        },
        |chunk| {
            emit_constant(chunk, Value::Str(("started".to_string()).into()));
            chunk.emit(Op::PrintLn, loc());
            emit_constant(chunk, Value::Integer(2_000));
            chunk.emit(
                Op::Intrinsic(u16::from(Intrinsic::Time(TimeIntrinsic::Sleep))),
                loc(),
            );
            emit_constant(chunk, Value::Str(("late".to_string()).into()));
            chunk.emit(Op::PrintLn, loc());
            chunk.emit(Op::Unit, loc());
            chunk.emit(Op::Return, loc());
        },
    );

    let mut vm = Vm::new(chunk);
    vm.set_worker_pool_size_for_tests(1);
    let started = Instant::now();
    vm.run()
        .expect("normal main completion should cancel sleeping detached work");

    assert!(
        started.elapsed() < Duration::from_millis(1_000),
        "VM teardown must not wait for a detached sleeper"
    );
    assert_eq!(vm.output().lines, vec!["started"]);
}
