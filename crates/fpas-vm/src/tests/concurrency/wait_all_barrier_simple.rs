//! Empty `WaitAll` and two-task barrier with per-task `Wait` + print.
//!
//! **Documentation:** `docs/rust/parallel-vm.md` (Phase 8), `docs/pascal/std/task.md`, `docs/pascal/08-concurrency.md`

use crate::Vm;
use fpas_bytecode::{Chunk, Intrinsic, Op, TaskIntrinsic, Value};

use crate::tests::helpers::{build_zero_arg_function_chunk, emit_constant, loc, run_ok_output};

// --- Positive: WaitAll ---

#[test]
fn wait_all_empty_array_completes_immediately() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::MakeArray(0), loc());
    chunk.emit(
        Op::Intrinsic(u16::from(Intrinsic::Task(TaskIntrinsic::WaitAll))),
        loc(),
    );
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
            chunk.emit(
                Op::Intrinsic(u16::from(Intrinsic::Task(TaskIntrinsic::WaitAll))),
                loc(),
            );
            chunk.emit(
                Op::Intrinsic(u16::from(Intrinsic::Task(TaskIntrinsic::Wait))),
                loc(),
            );
            chunk.emit(Op::PrintLn, loc());
            chunk.emit(
                Op::Intrinsic(u16::from(Intrinsic::Task(TaskIntrinsic::Wait))),
                loc(),
            );
            chunk.emit(Op::PrintLn, loc());
        },
        |chunk| {
            emit_constant(chunk, Value::Integer(7));
            chunk.emit(Op::Return, loc());
        },
    );

    assert_eq!(run_ok_output(chunk), vec!["7", "7"]);
}
