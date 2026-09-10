//! Task-group failure collection must agree in normal and deterministic execution.

use super::*;
use crate::DebugTaskEventKind;

mod timeout;

const LIFECYCLE: &str =
    include_str!("../../../../../../tests/concurrency/task_group_lifecycle_test.fpas");

const SOURCE: &str = r#"program GroupFailures;
uses Std.Task, Std.Time, Std.Arrays;
function Ordinary(Token: CancellationToken): result of integer, string;
begin
  Sleep(1);
  return Error('ordinary')
end;
procedure Broken(Token: CancellationToken);
begin
  Sleep(2);
  panic('owned panic')
end;
function Successful(Token: CancellationToken): integer;
begin return 42 end;
begin
  var G: TaskGroup := CreateTaskGroup();
  var A: task := StartTaskInGroup(G, Ordinary);
  var B: task := StartTaskInGroup(G, Broken);
  var C: task := StartTaskInGroup(G, Successful);
  if Wait(C) <> 42 then panic('child value');
  var Failures: array of TaskFailure := CloseTaskGroup(G);
  if Length(Failures) <> 2 then panic('failure count');
  if Failures[0].Kind <> TaskFailureKind.ReturnedError then panic('ordinary kind');
  if Failures[0].Message <> 'ordinary' then panic('ordinary message');
  if Failures[1].Kind <> TaskFailureKind.Panicked then panic('panic kind');
  if Failures[1].Line <= 0 then panic('panic location');
  if Length(CloseTaskGroup(G)) <> 0 then panic('repeat close')
end."#;

#[test]
fn task_group_start_activates_default_workers_and_timers() {
    let (program, errors) = fpas_parser::parse(SOURCE);
    assert!(errors.is_empty(), "{errors:?}");
    let mut vm = crate::vm::Vm::new(fpas_compiler::compile(&program).expect("compile"));
    assert!(
        vm.pool_size > 0,
        "group start requires workers and timer driver"
    );
    vm.run().expect("group failure report with default pool");
}

#[test]
fn task_group_failures_are_contained_with_one_worker() {
    let (program, errors) = fpas_parser::parse(SOURCE);
    assert!(errors.is_empty(), "{errors:?}");
    let mut vm = crate::vm::Vm::new(fpas_compiler::compile(&program).expect("compile"));
    vm.pool_size = 1;
    vm.run().expect("group failure report");
}

#[test]
fn task_group_failures_do_not_stop_the_debugger_parent() {
    let (program, errors) = fpas_parser::parse(SOURCE);
    assert!(errors.is_empty(), "{errors:?}");
    let mut session =
        DebugSession::with_manual_clock(fpas_compiler::compile(&program).expect("compile"))
            .expect("session");
    assert!(matches!(
        session.continue_execution().expect("run"),
        DebugRunResult::Terminated(_)
    ));
}

#[test]
fn task_group_nested_children_observe_cancellation_with_one_worker() {
    let (program, errors) = fpas_parser::parse(LIFECYCLE);
    assert!(errors.is_empty(), "{errors:?}");
    let mut vm = crate::vm::Vm::new(fpas_compiler::compile(&program).expect("compile"));
    vm.pool_size = 1;
    vm.run().expect("nested child cancellation");
}

#[test]
fn task_group_nested_children_observe_cancellation_in_debugger() {
    let (program, errors) = fpas_parser::parse(LIFECYCLE);
    assert!(errors.is_empty(), "{errors:?}");
    let mut session =
        DebugSession::with_manual_clock(fpas_compiler::compile(&program).expect("compile"))
            .expect("session");
    assert!(matches!(
        session.continue_execution().expect("run"),
        DebugRunResult::Terminated(_)
    ));
}

fn stop_before_close(source: &str) -> DebugSession {
    let source = source
        .replace(
            "\nbegin\n  var G:",
            "\nprocedure BeforeClose(); begin Sleep(0) end;\nbegin\n  var G:",
        )
        .replace("  var Failures:", "  BeforeClose();\n  var Failures:");
    let (program, errors) = fpas_parser::parse(&source);
    assert!(errors.is_empty(), "{errors:?}");
    let mut session =
        DebugSession::with_manual_clock(fpas_compiler::compile(&program).expect("compile"))
            .expect("session");
    let bound = session
        .replace_function_breakpoints(vec![FunctionBreakpoint {
            name: "BeforeClose".into(),
        }])
        .expect("breakpoint");
    assert!(bound[0].is_verified());
    let stop = stopped(session.continue_execution().expect("stop before close"));
    assert_eq!(stop.reason, DebugStopReason::Breakpoint);
    assert_eq!(stop.task_id, 0);
    session
}
#[test]
fn task_group_exited_failure_cannot_be_resumed_or_force_returned() {
    let source = SOURCE.replace(
        "  var Failures:",
        "  Select([TimerCase(5, procedure() begin end)]);\n  var Failures:",
    );
    let mut session = stop_before_close(&source);
    let events = session.take_task_events();
    assert_eq!(
        events
            .iter()
            .filter(|event| event.task_id == 2 && event.kind == DebugTaskEventKind::Exited)
            .count(),
        1,
        "events: {events:?}; tasks: {:?}; stop: {:?}",
        session.tasks(0, 8).expect("tasks"),
        session.last_stop()
    );
    let child = session
        .tasks(0, 8)
        .expect("tasks")
        .items
        .into_iter()
        .find(|task| task.id == 2)
        .expect("failed child");
    assert_eq!(child.state, DebugTaskState::Failed);
    let frame = session.stack_for_task(2, 0, 1).expect("failed stack").items[0].id;
    let before = session.test_instruction_count();
    assert!(session.resume_task(2).is_err());
    assert!(session.force_return(frame, None).is_err());
    assert_eq!(session.test_instruction_count(), before);
    assert!(matches!(
        session
            .continue_execution()
            .expect("close after rejected recovery"),
        DebugRunResult::Terminated(_)
    ));
}

#[test]
fn task_group_debugger_cancellation_is_reported_as_cancelled() {
    let source = r#"program CancelOwnedChild;
uses Std.Task, Std.Time, Std.Arrays;
procedure Work(Token: CancellationToken); begin Sleep(1000) end;
begin
  var G: TaskGroup := CreateTaskGroup();
  StartTaskInGroup(G, Work);
  var Failures: array of TaskFailure := CloseTaskGroup(G);
  if Length(Failures) <> 1 then panic('failure count');
  if Failures[0].Kind <> TaskFailureKind.Cancelled then panic('cancellation kind')
end."#;
    let mut session = stop_before_close(source);
    session.cancel_task(1).expect("cancel owned child");
    assert!(matches!(
        session
            .continue_execution()
            .expect("close after cancellation"),
        DebugRunResult::Terminated(_)
    ));
}

#[test]
fn task_group_child_close_is_rejected_and_collected_as_runtime_failure() {
    let source = r#"program ChildCannotClose;
uses Std.Task, Std.Arrays;
begin
  var G: TaskGroup := CreateTaskGroup();
  StartTaskInGroup(G, procedure(Token: CancellationToken) begin CloseTaskGroup(G) end);
  var Failures: array of TaskFailure := CloseTaskGroup(G);
  if Length(Failures) <> 1 then panic('failure count');
  if Failures[0].Kind <> TaskFailureKind.RuntimeError then panic('runtime kind')
end."#;
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    let executable = fpas_compiler::compile(&program).expect("compile");
    let mut vm = crate::vm::Vm::new(executable.clone());
    vm.pool_size = 1;
    vm.run().expect("child close rejected without self-join");
    let mut session = DebugSession::with_manual_clock(executable).expect("session");
    assert!(matches!(
        session.continue_execution().expect("debug child close"),
        DebugRunResult::Terminated(_)
    ));
}

#[test]
fn task_group_explicit_wait_keeps_the_original_panic_diagnostic() {
    let source = r#"program ExplicitFailedWait;
uses Std.Task;
procedure Work(Token: CancellationToken); begin panic('original child failure') end;
begin
  var G: TaskGroup := CreateTaskGroup();
  var Child: task := StartTaskInGroup(G, Work);
  Wait(Child)
end."#;
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    let mut vm = crate::vm::Vm::new(fpas_compiler::compile(&program).expect("compile"));
    vm.pool_size = 1;
    let error = vm.run().expect_err("explicit wait must propagate panic");
    assert_eq!(error.code, fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC);
    assert_eq!(error.message, "panic: original child failure");
}

#[test]
fn task_group_ignored_close_report_still_has_verified_record_metadata() {
    let source = r#"program IgnoreReport;
uses Std.Task;
procedure Work(Token: CancellationToken); begin panic('contained') end;
begin
  var G: TaskGroup := CreateTaskGroup();
  StartTaskInGroup(G, Work);
  CloseTaskGroup(G)
end."#;
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    let mut vm = crate::vm::Vm::new(fpas_compiler::compile(&program).expect("compile"));
    vm.run().expect("discarded report");
}
