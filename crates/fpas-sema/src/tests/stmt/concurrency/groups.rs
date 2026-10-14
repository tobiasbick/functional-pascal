//! Static contracts for explicitly owned task workers.

use super::{SEMA_TASK_BOUND_CALLABLE, check_errors, check_ok};

#[test]
fn group_operations_reject_wrong_handles_and_worker_signatures() {
    for call in [
        "Tasks.CreateTaskGroup(1)",
        "Tasks.GetTaskGroupToken(CreateCancellationSource())",
        "Tasks.CancelTaskGroup(GetCancellationToken(CreateCancellationSource()))",
        "Tasks.CloseTaskGroup(1)",
        "CloseTaskGroupWithTimeout(1, 0)",
        "CloseTaskGroupWithTimeout(G, true)",
        "CloseTaskGroupWithTimeout(G)",
        "CloseTaskGroupWithTimeout(G, 0, 0)",
        "TryCloseCompletedTaskGroup(1)",
        "TryCloseCompletedTaskGroup(G, 0)",
        "Tasks.StartTaskInGroup(G)",
        "Tasks.StartTaskInGroup(1, Work)",
        "Tasks.StartTaskInGroup(G, 1)",
        "Tasks.StartSupervisedTask(G, Work, true, 0)",
        "Tasks.StartSupervisedTask(G, Work, 1, 'bad')",
        "Tasks.StartSupervisedTask(G, Work, 1)",
        "Tasks.StartSupervisedTask(G, procedure() begin end procedure, 1, 0)",
        "Tasks.StartTaskInGroup(G, procedure() begin end procedure)",
        "Tasks.StartTaskInGroup(G, procedure(T: integer) begin end procedure)",
        "Tasks.StartTaskInGroup(G, procedure(T: CancellationSource) begin end procedure)",
        "Tasks.StartTaskInGroup(G, procedure(T: Tasks.CancellationToken; Extra: integer) begin end procedure)",
    ] {
        let source = format!(
            "program T; uses Std.Tasks as Tasks; procedure Work(Token: Tasks.CancellationToken); begin end procedure; begin const G: Tasks.TaskGroup := Tasks.CreateTaskGroup(); {call}; end."
        );
        assert!(
            !check_errors(&source).is_empty(),
            "accepted invalid group operation: {call}"
        );
    }
}

#[test]
fn group_worker_rejects_mutable_captures() {
    let errors = check_errors(
        r#"program T;
uses Std.Tasks as Tasks;
begin
  var Count: integer := 0;
  const G: Tasks.TaskGroup := Tasks.CreateTaskGroup();
  Tasks.StartTaskInGroup(G, procedure(Token: Tasks.CancellationToken) begin Count := Count + 1; end procedure);
end."#,
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_TASK_BOUND_CALLABLE),
        "{errors:?}"
    );
}

#[test]
fn group_worker_preserves_unit_value_and_result_task_types() {
    check_ok(
        r#"program T;
uses Std.Tasks as Tasks;
procedure NoValue(Token: Tasks.CancellationToken); begin end procedure;
function Number(Token: Tasks.CancellationToken): integer; begin return 42; end function;
function Outcome(Token: Tasks.CancellationToken): result of (integer, string); begin return Error('failed'); end function;
begin
  const G: Tasks.TaskGroup := Tasks.CreateTaskGroup();
  const A: task := Tasks.StartTaskInGroup(G, NoValue);
  const B: task of integer := Tasks.StartTaskInGroup(G, Number);
  const C: task of result of (integer, string) := Tasks.StartTaskInGroup(G, Outcome);
  Tasks.Wait(A);
  const N: integer := Tasks.Wait(B);
  const R: result of (integer, string) := Tasks.Wait(C);
  discard Tasks.CloseTaskGroup(G);
end."#,
    );
}

#[test]
fn group_worker_does_not_erase_its_result_type() {
    let errors = check_errors(
        r#"program T;
uses Std.Tasks as Tasks;
function Work(Token: Tasks.CancellationToken): integer; begin return 7; end function;
begin
  const G: Tasks.TaskGroup := Tasks.CreateTaskGroup();
  const Child: task of integer := Tasks.StartTaskInGroup(G, Work);
  const Wrong: string := Tasks.Wait(Child);
end."#,
    );
    assert!(!errors.is_empty(), "worker result was erased");
}

#[test]
fn timed_group_close_preserves_result_and_failure_record_types() {
    check_ok(
        "program T; uses Std.Tasks as Tasks; begin const G: Tasks.TaskGroup := Tasks.CreateTaskGroup(); const R: result of (array of Tasks.TaskFailure, string) := Tasks.CloseTaskGroupWithTimeout(G, 0); const Reports: array of Tasks.TaskFailure := R.Unwrap(); end.",
    );
    assert!(!check_errors("program T; uses Std.Tasks as Tasks; begin const G: Tasks.TaskGroup := Tasks.CreateTaskGroup(); const Wrong: boolean := Tasks.CloseTaskGroupWithTimeout(G, 0); end.").is_empty());
}

#[test]
fn completed_group_probe_preserves_optional_failure_report_type() {
    check_ok(
        "program T; uses Std.Tasks as Tasks; begin const G: Tasks.TaskGroup := Tasks.CreateTaskGroup(); const R: option of array of Tasks.TaskFailure := Tasks.TryCloseCompletedTaskGroup(G); end.",
    );
    assert!(
        !check_errors(
            "program T; uses Std.Tasks as Tasks; begin const G: Tasks.TaskGroup := Tasks.CreateTaskGroup(); const Wrong: boolean := Tasks.TryCloseCompletedTaskGroup(G); end."
        )
        .is_empty()
    );
}
