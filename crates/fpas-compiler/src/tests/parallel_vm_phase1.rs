//! Phase 1 from `docs/rust/parallel-vm.md`: emitted spawn opcodes and `Chunk::uses_spawn_tasks`.
//!
//! **Documentation:** `docs/rust/parallel-vm.md` (Phase 1), `docs/pascal/08-concurrency.md`

use super::compile_ok;
use fpas_bytecode::Op;

fn spawn_task_ops(chunk: &fpas_bytecode::Chunk) -> usize {
    chunk
        .code
        .iter()
        .filter(|op| matches!(op, Op::SpawnTask(_)))
        .count()
}

fn spawn_detached_ops(chunk: &fpas_bytecode::Chunk) -> usize {
    chunk
        .code
        .iter()
        .filter(|op| matches!(op, Op::SpawnDetachedTask(_)))
        .count()
}

// --- Positive ---

#[test]
fn go_expression_emits_spawn_task_and_marks_chunk() {
    let chunk = compile_ok(
        "\
program T;
uses Std.Console, Std.Task;

function Work(): integer;
begin
  return 5
end;

begin
  var T: task := go Work();
  Std.Console.WriteLn(Std.Task.Wait(T))
end.",
    );

    assert!(chunk.uses_spawn_tasks());
    assert!(spawn_task_ops(&chunk) >= 1);
    assert_eq!(spawn_detached_ops(&chunk), 0);
    assert!(
        chunk.code.iter().any(|op| matches!(op, Op::SpawnTask(0))),
        "expected zero-arg spawn, got: {:?}",
        chunk.code
    );
}

#[test]
fn go_statement_emits_spawn_detached_task_and_marks_chunk() {
    let chunk = compile_ok(
        "\
program T;
uses Std.Console;

procedure Side();
begin
  Std.Console.WriteLn('side')
end;

begin
  go Side()
end.",
    );

    assert!(chunk.uses_spawn_tasks());
    assert!(spawn_detached_ops(&chunk) >= 1);
    assert_eq!(spawn_task_ops(&chunk), 0);
    assert!(
        chunk
            .code
            .iter()
            .any(|op| matches!(op, Op::SpawnDetachedTask(0))),
        "expected zero-arg detached spawn"
    );
}

#[test]
fn go_with_two_arguments_emits_spawn_task_with_argc_two() {
    let chunk = compile_ok(
        "\
program T;
uses Std.Console, Std.Task;

function Add(A: integer; B: integer): integer;
begin
  return A + B
end;

begin
  var T: task := go Add(10, 20);
  Std.Console.WriteLn(Std.Task.Wait(T))
end.",
    );

    assert!(chunk.uses_spawn_tasks());
    assert!(
        chunk.code.iter().any(|op| matches!(op, Op::SpawnTask(2))),
        "expected SpawnTask(2), got: {:?}",
        chunk.code
    );
}

#[test]
fn program_with_both_go_forms_contains_both_spawn_opcodes() {
    let chunk = compile_ok(
        "\
program T;
uses Std.Console, Std.Task;

procedure Side();
begin
  Std.Console.WriteLn('s')
end;

function N(): integer;
begin
  return 1
end;

begin
  go Side();
  var T: task := go N();
  Std.Console.WriteLn(Std.Task.Wait(T))
end.",
    );

    assert!(chunk.uses_spawn_tasks());
    assert!(spawn_task_ops(&chunk) >= 1);
    assert!(spawn_detached_ops(&chunk) >= 1);
}

// --- Negative ---

#[test]
fn program_without_go_does_not_mark_uses_spawn_tasks() {
    let chunk = compile_ok(
        "\
program T;
uses Std.Console;

begin
  Std.Console.WriteLn('hello')
end.",
    );

    assert!(!chunk.uses_spawn_tasks());
    assert_eq!(spawn_task_ops(&chunk), 0);
    assert_eq!(spawn_detached_ops(&chunk), 0);
}

#[test]
fn importing_std_task_without_go_does_not_mark_chunk() {
    let chunk = compile_ok(
        "\
program T;
uses Std.Console, Std.Task;

begin
  Std.Console.WriteLn('no spawn')
end.",
    );

    assert!(!chunk.uses_spawn_tasks());
}

// --- Edge ---

#[test]
fn go_std_call_wrapper_still_emits_spawn_opcode() {
    let chunk = compile_ok(
        "\
program T;
uses Std.Console, Std.Task, Std.Conv;

begin
  var T: task := go Std.Conv.IntToStr(7);
  Std.Console.WriteLn(Std.Task.Wait(T))
end.",
    );

    assert!(chunk.uses_spawn_tasks());
    assert!(
        spawn_task_ops(&chunk) >= 1 || spawn_detached_ops(&chunk) >= 1,
        "wrapper lowering should still leave a spawn opcode in bytecode"
    );
}
