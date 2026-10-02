//! Static contracts for explicitly owned task workers.

use super::{SEMA_TASK_BOUND_CALLABLE, check_errors, check_ok};

#[test]
fn group_operations_reject_wrong_handles_and_worker_signatures() {
    for call in [
        "Workers.CreateTaskGroup(1)",
        "Workers.GetTaskGroupToken(Workers.CreateWorkers.CancellationSource())",
        "Workers.CancelTaskGroup(Workers.GetWorkers.CancellationToken(Workers.CreateWorkers.CancellationSource()))",
        "Workers.CloseTaskGroup(1)",
        "Workers.CloseTaskGroupWithTimeout(1, 0)",
        "Workers.CloseTaskGroupWithTimeout(G, true)",
        "Workers.CloseTaskGroupWithTimeout(G)",
        "Workers.CloseTaskGroupWithTimeout(G, 0, 0)",
        "Workers.TryCloseCompletedTaskGroup(1)",
        "Workers.TryCloseCompletedTaskGroup(G, 0)",
        "Workers.StartTaskInGroup(G)",
        "Workers.StartTaskInGroup(1, Work)",
        "Workers.StartTaskInGroup(G, 1)",
        "Workers.StartSupervisedTask(G, Work, true, 0)",
        "Workers.StartSupervisedTask(G, Work, 1, 'bad')",
        "Workers.StartSupervisedTask(G, Work, 1)",
        "Workers.StartSupervisedTask(G, procedure() begin null; end procedure, 1, 0)",
        "Workers.StartTaskInGroup(G, procedure() begin null; end procedure)",
        "Workers.StartTaskInGroup(G, procedure(T: integer) begin null; end procedure)",
        "Workers.StartTaskInGroup(G, procedure(T: Workers.CancellationSource) begin null; end procedure)",
        "Workers.StartTaskInGroup(G, procedure(T: Workers.CancellationToken; Extra: integer) begin null; end procedure)",
        "Workers.StartTaskInGroup(G, function(T: Workers.CancellationToken): result of integer, integer begin return Error(1); end function)",
    ] {
        let source = format!(
            "program T; uses Std.Tasks as Workers; procedure Work(Token: Workers.CancellationToken); begin null; end procedure; begin var G: Workers.TaskGroup := Workers.CreateTaskGroup(); {call}; end program;"
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
  mutable var Count: integer := 0;
  var G: Tasks.TaskGroup := Tasks.CreateTaskGroup();
  Tasks.StartTaskInGroup(G, procedure(Token: Tasks.CancellationToken) begin Count := Count + 1; end procedure);
end program;"#,
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
procedure NoValue(Token: Tasks.CancellationToken); begin null; end procedure;
function Number(Token: Tasks.CancellationToken): integer; begin return 42; end function;
function Outcome(Token: Tasks.CancellationToken): result of integer, string; begin return Error('failed'); end function;
begin
  var G: Tasks.TaskGroup := Tasks.CreateTaskGroup();
  var A: task := Tasks.StartTaskInGroup(G, NoValue);
  var B: task := Tasks.StartTaskInGroup(G, Number);
  var C: task := Tasks.StartTaskInGroup(G, Outcome);
  Tasks.Wait(A);
  var N: integer := Tasks.Wait(B);
  var R: result of integer, string := Tasks.Wait(C);
  Tasks.CloseTaskGroup(G);
end program;"#,
    );
}

#[test]
fn group_worker_does_not_erase_its_result_type() {
    let errors = check_errors(
        r#"program T;
uses Std.Tasks as Tasks;
function Work(Token: Tasks.CancellationToken): integer; begin return 7; end function;
begin
  var G: Tasks.TaskGroup := Tasks.CreateTaskGroup();
  var Child: task := Tasks.StartTaskInGroup(G, Work);
  var Wrong: string := Tasks.Wait(Child);
end program;"#,
    );
    assert!(!errors.is_empty(), "worker result was erased");
}

#[test]
fn timed_group_close_preserves_result_and_failure_record_types() {
    check_ok(
        r#"program T;  uses Std.Tasks as Tasks; uses Std.Results as Results; begin var G: Tasks.TaskGroup := Tasks.CreateTaskGroup(); var R: result of array of Tasks.TaskFailure, string := Tasks.CloseTaskGroupWithTimeout(G, 0); var Reports: array of Tasks.TaskFailure := Results.Unwrap(R); end program;"#,
    );
    assert!(!check_errors(r#"program T;  uses Std.Tasks as Tasks; begin var G: Tasks.TaskGroup := Tasks.CreateTaskGroup(); var Wrong: boolean := Tasks.CloseTaskGroupWithTimeout(G, 0); end program;"#).is_empty());
}

#[test]
fn completed_group_probe_preserves_optional_failure_report_type() {
    check_ok(
        r#"program T;  uses Std.Tasks as Tasks; begin var G: Tasks.TaskGroup := Tasks.CreateTaskGroup(); var R: option of array of Tasks.TaskFailure := Tasks.TryCloseCompletedTaskGroup(G); end program;"#,
    );
    assert!(
        !check_errors(
            r#"program T;  uses Std.Tasks as Tasks; begin var G: Tasks.TaskGroup := Tasks.CreateTaskGroup(); var Wrong: boolean := Tasks.TryCloseCompletedTaskGroup(G); end program;"#
        )
        .is_empty()
    );
}
