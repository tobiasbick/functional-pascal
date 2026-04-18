//! Scoped [`Vm::run`] completion and integration with pool join.
//!
//! **Documentation:** `docs/rust/parallel-vm.md` (Phase 4 checklist), `docs/pascal/08-concurrency.md`

use crate::Vm;
use fpas_bytecode::{Chunk, Op, Value};

use crate::tests::helpers::{emit_constant, loc};

// --- Positive: scoped run completes and can be repeated ---

#[test]
fn run_with_zero_pool_workers_completes_halt_only_program() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Halt, loc());
    let mut vm = Vm::new(chunk);
    assert_eq!(vm.worker_pool_size_for_tests(), 0);
    vm.run().expect("halt-only run");
}

#[test]
fn repeated_vm_runs_with_zero_pool_each_complete() {
    for _ in 0..32 {
        let mut chunk = Chunk::new();
        chunk.emit(Op::Halt, loc());
        let mut vm = Vm::new(chunk);
        vm.run().expect("repeated short runs");
    }
}

// --- Positive: VM run joins pool after main (integration with real Vm) ---

#[test]
fn vm_run_with_spawn_chunk_returns_after_pool_workers_join() {
    let function_name = "Quick";
    let mut chunk = Chunk::new();
    emit_constant(
        &mut chunk,
        Value::Function {
            name: function_name.to_string(),
            captures: vec![],
        },
    );
    chunk.emit(Op::SpawnTask(0), loc());
    chunk.emit(Op::Halt, loc());

    let body_start = chunk.len();
    chunk
        .functions
        .insert(function_name.to_string(), (body_start, 0));
    emit_constant(&mut chunk, Value::Integer(1));
    chunk.emit(Op::Return, loc());

    assert!(chunk.uses_spawn_tasks());
    assert!(Vm::new(chunk.clone()).worker_pool_size_for_tests() > 0);

    let mut vm = Vm::new(chunk);
    vm.run().expect("main halts; pool workers idle then join");
}
