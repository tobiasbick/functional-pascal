//! Nested task-bound closure captures and sync-callback Wait progress.
//!
//! **Documentation:** `docs/pascal/language/functions/closures.md`,
//! `docs/pascal/std/concurrency/task.md`

use fpas_bytecode::{ArrayIntrinsic, Intrinsic, Op, TaskIntrinsic, Value};
use fpas_diagnostics::codes::RUNTIME_INVALID_TASK;

use crate::Vm;
use crate::tests::helpers::{
    build_function_chunk, build_zero_arg_function_chunk, emit_constant, loc, run_err, run_ok_output,
};

#[test]
fn make_closure_propagates_task_bound_from_nested_function_capture() {
    let outer = "Outer";
    let chunk = build_zero_arg_function_chunk(
        outer,
        |chunk| {
            // Capture a task-bound function value; MakeClosure must mark Outer task-bound.
            emit_constant(chunk, Value::function("Inner".to_string(), vec![], true));
            let name_idx = chunk
                .add_constant(Value::Str(outer.into()))
                .expect("constant fits");
            chunk.emit(Op::MakeClosure(name_idx, 1), loc());
            chunk.emit(Op::SpawnTask(0), loc());
        },
        |chunk| {
            emit_constant(chunk, Value::Integer(0));
            chunk.emit(Op::Return, loc());
        },
    );

    let err = run_err(chunk);
    assert_eq!(err.code, RUNTIME_INVALID_TASK);
    assert!(
        err.message.contains("task-bound"),
        "unexpected message: {}",
        err.message
    );
}

#[test]
fn spawn_rejects_function_value_marked_task_bound() {
    let mut chunk = fpas_bytecode::Chunk::new();
    emit_constant(
        &mut chunk,
        Value::function("Bound".to_string(), vec![], true),
    );
    chunk.emit(Op::SpawnTask(0), loc());
    chunk.emit(Op::Halt, loc());
    let code_start = chunk.len();
    chunk.insert_function("bound".to_string(), code_start, 0);
    emit_constant(&mut chunk, Value::Integer(0));
    chunk.emit(Op::Return, loc());

    let err = run_err(chunk);
    assert_eq!(err.code, RUNTIME_INVALID_TASK);
}

fn map_spawn_wait_chunk(
    input: i64,
    spawn_and_wait: &str,
    double_later: &str,
) -> fpas_bytecode::Chunk {
    build_function_chunk(
        spawn_and_wait,
        1,
        |chunk| {
            emit_constant(chunk, Value::Array(vec![Value::Integer(input)].into()));
            emit_constant(
                chunk,
                Value::function(spawn_and_wait.to_string(), vec![], false),
            );
            chunk.emit(
                Op::Intrinsic(u16::from(Intrinsic::Array(ArrayIntrinsic::Map))),
                loc(),
            );
            emit_constant(chunk, Value::Integer(0));
            chunk.emit(Op::IndexGet, loc());
            chunk.emit(Op::PrintLn, loc());
            chunk.emit(Op::Halt, loc());

            let start = chunk.len();
            chunk.insert_function(double_later.to_string(), start, 1);
            chunk.emit(Op::GetLocal(0), loc());
            emit_constant(chunk, Value::Integer(2));
            chunk.emit(Op::MulInt, loc());
            chunk.emit(Op::Return, loc());
        },
        |chunk| {
            chunk.emit(Op::GetLocal(0), loc());
            emit_constant(
                chunk,
                Value::function(double_later.to_string(), vec![], false),
            );
            chunk.emit(Op::SpawnTask(1), loc());
            chunk.emit(
                Op::Intrinsic(u16::from(Intrinsic::Task(TaskIntrinsic::Wait))),
                loc(),
            );
            chunk.emit(Op::Return, loc());
        },
    )
}

#[test]
fn wait_inside_array_map_with_single_worker_completes() {
    // Regression: sync callbacks cannot yield; Wait must help-run ready tasks
    // so pool_size == 1 cannot livelock.
    let chunk = map_spawn_wait_chunk(7, "SpawnAndWait", "DoubleLater");
    let mut vm = Vm::new(chunk);
    vm.set_worker_pool_size_for_tests(1);
    vm.run()
        .expect("Wait inside Map must complete with one worker");
    assert_eq!(vm.output().lines, vec!["14"]);
}

#[test]
fn wait_inside_array_map_callback_still_works_with_default_pool() {
    let chunk = map_spawn_wait_chunk(3, "SpawnAndWait", "DoubleLater");
    assert_eq!(run_ok_output(chunk), vec!["6"]);
}
