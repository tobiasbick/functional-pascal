use super::*;

#[test]
fn controlled_wait_any_rejects_negative_timeout() {
    let error = run_program(
        r#"program BadTimeout;
uses Std.Task;
procedure Work();
begin
end;
begin
  var T: task := go Work();
  WaitAnyWithTimeout([T], -1)
end."#,
    )
    .expect_err("negative timeout");
    assert!(error.message.contains("non-negative timeout"));
}

#[test]
fn controlled_wait_any_preserves_worker_failure() {
    for control in [
        "WithTimeout([T], 1000)",
        "WithCancellation([T], GetCancellationToken(CreateCancellationSource()))",
    ] {
        let source = format!(
            "program Failure; uses Std.Task; procedure Work(); begin panic('original failure') end; begin var T: task := go Work(); WaitAny{control} end."
        );
        let error = run_program(&source).expect_err("task failure");
        assert_eq!(error.code, fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC);
        assert!(error.message.contains("original failure"));
    }
}

#[test]
fn wait_any_rejects_an_empty_task_array() {
    let error = run_program("program EmptyWaitAny; uses Std.Task; begin var Tasks: array of task := []; WaitAny(Tasks) end.").expect_err("empty list");
    assert_eq!(error.code, fpas_diagnostics::codes::RUNTIME_INVALID_TASK);
    assert!(error.message.contains("between 1 and 1048576"));
}

#[test]
fn wait_any_preserves_worker_failure_diagnostic() {
    let error = run_program(
        r#"program FailedWaitAny;
uses Std.Task;
procedure Work();
begin
  panic('original worker failure')
end;
begin
  var T: task := go Work();
  WaitAny([T])
end."#,
    )
    .expect_err("worker failure");
    assert_eq!(error.code, fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC);
    assert!(error.message.contains("original worker failure"));
}

#[test]
fn wait_any_preserves_results_and_array_order() {
    assert_succeeds(
        r#"program WaitAnyOrder;
uses Std.Task;
function Work(Value: integer): integer;
begin
  return Value
end;
begin
  var A: task := go Work(11);
  var B: task := go Work(22);
  WaitAll([A, B]);
  if WaitAny([B, A, B]) <> 0 then panic('wrong index');
  if Wait(B) <> 22 then panic('result consumed');
  if WaitAny([B, A]) <> 0 then panic('consumed completion lost');
  if Wait(A) <> 11 then panic('losing result consumed')
end."#,
    );
}

#[test]
fn wait_any_worker_helps_nested_tasks() {
    assert_succeeds(
        r#"program WaitAnyNested;
uses Std.Task, Std.Time;
function Work(): integer;
begin
  Sleep(1);
  return 7
end;
function Parent(): integer;
begin
  var Child: task := go Work();
  if WaitAny([Child]) <> 0 then panic('index');
  return Wait(Child)
end;
begin
  var ParentTask: task := go Parent();
  if WaitAny([ParentTask]) <> 0 then panic('parent index');
  if Wait(ParentTask) <> 7 then panic('value')
end."#,
    );
}
