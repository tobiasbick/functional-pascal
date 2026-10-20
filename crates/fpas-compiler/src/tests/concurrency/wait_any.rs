use super::*;

#[test]
fn controlled_wait_any_rejects_negative_timeout() {
    assert_succeeds(
        r#"program BadTimeout;
uses Std.Tasks as Tasks, Std.Test as Test;
procedure Work();
begin
end procedure;
begin scope
  const T: task := go Work();
  Test.AssertTrue(Tasks.WaitAnyWithTimeout([T], -1) is Error(Tasks.TaskError.InvalidArgument('TimeoutMillis', _)));
 end scope; end."#,
    );
}

#[test]
fn controlled_wait_any_preserves_worker_failure() {
    for control in [
        "WithTimeout([T], 1000)",
        "WithCancellation([T], Tasks.GetCancellationToken(Tasks.CreateCancellationSource()))",
    ] {
        let source = format!(
            "program Failure; uses Std.Tasks as Tasks; procedure Work(); begin panic('original failure'); end procedure; begin scope  const T: task := go Work(); discard Tasks.WaitAny{control};  end scope; end."
        );
        let error = run_program(&source).expect_err("task failure");
        assert_eq!(error.code, fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC);
        assert!(error.message.contains("original failure"));
    }
}

#[test]
fn wait_any_rejects_an_empty_task_array() {
    let error = run_program("program EmptyWaitAny; uses Std.Tasks as StdTasks; begin const Tasks: array of task := []; discard StdTasks.WaitAny(Tasks); end.").expect_err("empty list");
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
begin scope
  const T: task := go Work();
  discard Tasks.WaitAny([T]);
 end scope; end."#,
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
begin scope
  const A: task of integer := go Work(11);
  const B: task of integer := go Work(22);
  Tasks.WaitAll([A, B]);
  if Tasks.WaitAny([B, A, B]) <> 0 then panic('wrong index'); end if;
  if Tasks.Wait(B) <> 22 then panic('result consumed'); end if;
  if Tasks.WaitAny([B, A]) <> 0 then panic('consumed completion lost'); end if;
  if Tasks.Wait(A) <> 11 then panic('losing result consumed'); end if;
 end scope; end."#,
    );
}

#[test]
fn wait_any_worker_helps_nested_tasks() {
    assert_succeeds(
        r#"program WaitAnyNested;
uses Std.Tasks as Tasks, Std.Time as Time;
function Work(): integer;
begin
  Time.Sleep(1);
  return 7;
end function;
function Parent(): integer;
begin scope
  const Child: task of integer := go Work();
  if Tasks.WaitAny([Child]) <> 0 then panic('index'); end if;
  return Tasks.Wait(Child);
 end scope; end function;
begin scope
  const ParentTask: task of integer := go Parent();
  if Tasks.WaitAny([ParentTask]) <> 0 then panic('parent index'); end if;
  if Tasks.Wait(ParentTask) <> 7 then panic('value'); end if;
 end scope; end."#,
    );
}
