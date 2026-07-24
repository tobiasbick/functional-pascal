//! Spawn opcodes and `Chunk::uses_spawn_tasks` tracking (`docs/pascal/language/concurrency/README.md`).
//!
//! **Documentation:** `docs/pascal/language/concurrency/README.md` (Phase 1), `docs/pascal/language/concurrency/README.md`

use fpas_bytecode::{Chunk, Op, SourceLocation, Value};

fn loc() -> SourceLocation {
    SourceLocation::new(1, 1)
}

// --- Positive: tracking returns true when a spawn opcode is present ---

#[test]
fn detects_spawn_task_with_zero_arity() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::SpawnTask(0), loc());
    assert!(chunk.uses_spawn_tasks());
}

#[test]
fn detects_spawn_task_with_max_u8_arity() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::SpawnTask(u8::MAX), loc());
    assert!(chunk.uses_spawn_tasks());
}

#[test]
fn detects_spawn_detached_task() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::SpawnDetachedTask(3), loc());
    assert!(chunk.uses_spawn_tasks());
}

#[test]
fn detects_spawn_after_many_non_spawn_ops() {
    let mut chunk = Chunk::new();
    for _ in 0..500 {
        chunk.emit(Op::Pop, loc());
        chunk.emit(Op::Dup, loc());
        chunk.emit(Op::AddInt, loc());
    }
    chunk.emit(Op::SpawnDetachedTask(0), loc());
    assert!(chunk.uses_spawn_tasks());
}

#[test]
fn detects_spawn_at_first_instruction() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::SpawnTask(1), loc());
    chunk.emit(Op::Unit, loc());
    chunk.emit(Op::Halt, loc());
    assert!(chunk.uses_spawn_tasks());
}

#[test]
fn detects_both_spawn_variants_in_one_chunk() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::SpawnTask(0), loc());
    chunk.emit(Op::SpawnDetachedTask(0), loc());
    assert!(chunk.uses_spawn_tasks());
}

#[test]
fn spawn_detection_is_independent_of_functions_table() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Constant(0), loc());
    chunk.emit(Op::Halt, loc());
    chunk.insert_function("unused".to_string(), 999, 0);
    assert!(
        !chunk.uses_spawn_tasks(),
        "only emitted spawn opcodes may enable the worker pool"
    );
}

// --- Negative: no spawn opcodes ---

#[test]
fn empty_chunk_does_not_use_spawn_tasks() {
    let chunk = Chunk::new();
    assert!(!chunk.uses_spawn_tasks());
}

#[test]
fn yield_only_does_not_use_spawn_tasks() {
    let mut chunk = Chunk::new();
    for _ in 0..20 {
        chunk.emit(Op::Yield, loc());
    }
    assert!(
        !chunk.uses_spawn_tasks(),
        "Phase 1 tracking is spawn-only; `Yield` must not imply a worker pool"
    );
}

#[test]
fn typical_control_flow_without_spawn() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Jump(3), loc());
    chunk.emit(Op::Pop, loc());
    chunk.emit(Op::JumpIfFalse(2), loc());
    chunk.emit(Op::Halt, loc());
    assert!(!chunk.uses_spawn_tasks());
}

#[test]
fn intrinsic_only_chunk_does_not_use_spawn_tasks() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Intrinsic(0), loc());
    chunk.emit(Op::Halt, loc());
    assert!(!chunk.uses_spawn_tasks());
}

// --- Edge cases ---

#[test]
fn yield_mixed_with_spawn_still_true() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Yield, loc());
    chunk.emit(Op::SpawnTask(0), loc());
    chunk.emit(Op::Yield, loc());
    assert!(chunk.uses_spawn_tasks());
}

#[test]
fn spawn_task_arity_differences_all_detected() {
    for argc in [0u8, 1, 7, 42, u8::MAX] {
        let mut chunk = Chunk::new();
        chunk.emit(Op::SpawnTask(argc), loc());
        assert!(chunk.uses_spawn_tasks(), "argc={argc}");
    }
}

#[test]
fn constant_pool_and_locations_do_not_affect_spawn_tracking() {
    let mut chunk = Chunk::new();
    assert!(chunk.add_constant(Value::Integer(1)).is_ok());
    chunk.emit(Op::Constant(0), loc());
    chunk.emit(Op::SpawnDetachedTask(0), loc());
    assert!(chunk.validate_invariants().is_ok());
    assert!(chunk.uses_spawn_tasks());
}

#[test]
fn default_chunk_matches_new() {
    assert!(!Chunk::default().uses_spawn_tasks());
}
