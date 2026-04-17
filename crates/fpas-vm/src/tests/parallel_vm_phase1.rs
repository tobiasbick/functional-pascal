//! Phase 1: `Chunk::uses_spawn_tasks` drives whether the VM constructs a worker pool at build time.
//!
//! **Documentation:** `docs/future/parallel-vm.md` (Phase 1), `docs/pascal/08-concurrency.md`

use crate::Vm;
use fpas_bytecode::{Chunk, Op};

use super::helpers::loc;

#[test]
fn vm_accepts_chunk_with_uses_spawn_tasks_false_without_running_spawn_path() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Halt, loc());
    assert!(!chunk.uses_spawn_tasks());

    let mut vm = Vm::new(chunk);
    vm.run()
        .expect("halt-only program should complete with zero pool workers");
}

#[test]
fn uses_spawn_tasks_true_does_not_change_chunk_scan_semantics_after_clone() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::SpawnTask(0), loc());
    assert!(chunk.uses_spawn_tasks());
    let copy = chunk.clone();
    assert!(copy.uses_spawn_tasks());
}
