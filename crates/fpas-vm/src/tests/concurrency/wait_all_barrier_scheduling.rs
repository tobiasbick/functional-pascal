//! Busy multi-spawn `WaitAll` barrier and main `Yield` between spawns before `WaitAll`.
//!
//! **Documentation:** `docs/pascal/08-concurrency.md` (Phase 8), `docs/pascal/std/task.md`, `docs/pascal/08-concurrency.md`

use crate::Vm;
use fpas_bytecode::{Intrinsic, Op, TaskIntrinsic, Value};

use crate::tests::helpers::{build_zero_arg_function_chunk, emit_constant, loc};

use super::fixtures::emit_instruction_waste;

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
            chunk.emit(
                Op::Intrinsic(u16::from(Intrinsic::Task(TaskIntrinsic::WaitAll))),
                loc(),
            );
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
            chunk.emit(
                Op::Intrinsic(u16::from(Intrinsic::Task(TaskIntrinsic::WaitAll))),
                loc(),
            );
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
