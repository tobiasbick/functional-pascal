use super::*;

#[test]
fn controlled_wait_any_rejects_negative_timeout() {
    let error = run_program(
        r#"program BadTimeout;
uses Std.Tasks as Tasks;
procedure Work();
begin null;
end procedure;
begin
  var T: task := go Work();
  discard Tasks.WaitAnyWithTimeout([T], -1);
end program;"#,
    )
    .expect_err("negative timeout");
    assert!(error.message.contains("non-negative timeout"));
}

#[test]
fn controlled_wait_any_preserves_worker_failure() {
    for control in [
        "WithTimeout([T], 1000)",
        "WithCancellation([T], Tasks.GetCancellationToken(Tasks.CreateCancellationSource()))",
    ] {
        let source = format!(
            "program Failure; uses Std.Tasks as Tasks; procedure Work(); begin panic('original failure'); end procedure; begin var T: task := go Work(); discard Tasks.WaitAny{control}; end program;"
        );
        let error = run_program(&source).expect_err("task failure");
        assert_eq!(error.code, fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC);
        assert!(error.message.contains("original failure"));
    }
}

#[test]
fn wait_any_rejects_an_empty_task_array() {
    let error = run_program(r#"program EmptyWaitAny;  uses Std.Tasks as Tasks2; begin var Tasks: array of task := []; discard Tasks2.WaitAny(Tasks); end program;"#).expect_err("empty list");
    assert_eq!(error.code, fpas_diagnostics::codes::RUNTIME_INVALID_TASK);
    assert!(error.message.contains("between 1 and 1048576"));
}

#[test]
fn wait_any_preserves_worker_failure_diagnostic() {
    let error = run_program(
        r#"program FailedWaitAny;
uses Std.Tasks as Tasks;
procedure Work();
begin
  panic('original worker failure');
end procedure;
begin
  var T: task := go Work();
  discard Tasks.WaitAny([T]);
end program;"#,
    )
    .expect_err("worker failure");
    assert_eq!(error.code, fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC);
    assert!(error.message.contains("original worker failure"));
}

#[test]
fn wait_any_preserves_results_and_array_order() {
    assert_succeeds(
        r#"program WaitAnyOrder;
uses Std.Tasks as Tasks;
function Work(Value: integer): integer;
begin
  return Value;
end function;
begin
  var A: task := go Work(11);
  var B: task := go Work(22);
  Tasks.WaitAll([A, B]);
  if Tasks.WaitAny([B, A, B]) <> 0 then panic('wrong index'); end if;
  if Tasks.Wait(B) <> 22 then panic('result consumed'); end if;
  if Tasks.WaitAny([B, A]) <> 0 then panic('consumed completion lost'); end if;
  if Tasks.Wait(A) <> 11 then panic('losing result consumed'); end if;
end program;"#,
    );
}

#[test]
fn wait_any_worker_helps_nested_tasks() {
    assert_succeeds(
        r#"program WaitAnyNested;
uses Std.Tasks as Tasks; uses Std.Time as Time;
function Work(): integer;
begin
  Time.Sleep(1);
  return 7;
end function;
function Parent(): integer;
begin
  var Child: task := go Work();
  if Tasks.WaitAny([Child]) <> 0 then panic('index'); end if;
  return Tasks.Wait(Child);
end function;
begin
  var ParentTask: task := go Parent();
  if Tasks.WaitAny([ParentTask]) <> 0 then panic('parent index'); end if;
  if Tasks.Wait(ParentTask) <> 7 then panic('value'); end if;
end program;"#,
    );
}
