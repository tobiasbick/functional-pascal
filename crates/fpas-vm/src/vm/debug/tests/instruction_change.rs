//! Instruction-change feasibility rejections.

use super::*;

const SOURCE: &str = r#"program InstructionChange;

uses Std.Console;

function Branch(Value: integer): integer;
begin
  mutable var Local: integer := Value + 10;
  WriteLn('effect');
  return Local
end;

begin
  WriteLn(Branch(1))
end.
"#;

fn session() -> DebugSession {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    DebugSession::new(fpas_compiler::compile(&program).expect("compile instruction-change fixture"))
        .expect("debug session")
}

fn stop_in_branch(session: &mut DebugSession) -> u64 {
    for _ in 0..128 {
        let stack = session.stack(0, 16).expect("stack");
        if stack
            .items
            .first()
            .is_some_and(|frame| frame.name == "branch")
        {
            return stack.items[0].id;
        }
        let _ = stopped(session.step_into().expect("step into branch"));
    }
    panic!("branch never became active")
}

#[test]
fn same_function_destinations_reject_without_changing_stopped_state() {
    let mut session = session();
    let frame = stop_in_branch(&mut session);
    let before = session.last_stop().clone();
    let instructions = session.test_instruction_count();
    let registers = session
        .test_worker_registers(before.task_id)
        .expect("stopped worker");
    let later = before.instruction.saturating_add(1);

    let error = session.set_instruction(Some(frame), Some(later));
    assert_eq!(error.kind, DebugErrorKind::InstructionChangeUnsupported);

    let current = session.set_instruction(Some(frame), Some(before.instruction));
    assert_eq!(current.kind, DebugErrorKind::InstructionChangeUnsupported);

    assert_eq!(session.last_stop(), &before);
    assert_eq!(session.test_instruction_count(), instructions);
    assert_eq!(
        session
            .test_worker_registers(before.task_id)
            .expect("unchanged worker"),
        registers
    );
    assert_eq!(
        session.stack(0, 1).expect("stack").items[0].id,
        frame,
        "inspection generation must stay valid"
    );
}

#[test]
fn stale_frames_fail_before_the_instruction_change_decision() {
    let mut session = session();
    let frame = stop_in_branch(&mut session);
    let _ = stopped(session.step_into().expect("advance"));
    let error = session.set_instruction(Some(frame), Some(0));
    assert_eq!(error.kind, DebugErrorKind::UnknownFrame);
}

#[test]
fn uninitialized_registers_keep_later_addresses_from_being_safe_destinations() {
    let mut session = session();
    let frame = stop_in_branch(&mut session);
    let stop = session.last_stop().clone();
    let registers = session
        .test_worker_registers(stop.task_id)
        .expect("live worker");
    assert!(
        registers.5.iter().any(|initialized| !initialized),
        "the live frame still has uninitialized registers that a later instruction may read"
    );

    let error = session.set_instruction(Some(frame), Some(stop.instruction.saturating_add(8)));
    assert_eq!(error.kind, DebugErrorKind::InstructionChangeUnsupported);
    assert_eq!(
        session
            .test_worker_registers(stop.task_id)
            .expect("still stopped"),
        registers
    );
}
