//! Static contracts for explicitly owned task workers.

use super::{SEMA_TASK_BOUND_CALLABLE, check_errors, check_ok};

#[test]
fn group_operations_reject_wrong_handles_and_worker_signatures() {
    for call in [
        "CreateTaskGroup(1)",
        "GetTaskGroupToken(CreateCancellationSource())",
        "CancelTaskGroup(GetCancellationToken(CreateCancellationSource()))",
        "CloseTaskGroup(1)",
        "CloseTaskGroupWithTimeout(1, 0)",
        "CloseTaskGroupWithTimeout(G, true)",
        "CloseTaskGroupWithTimeout(G)",
        "CloseTaskGroupWithTimeout(G, 0, 0)",
        "TryCloseCompletedTaskGroup(1)",
        "TryCloseCompletedTaskGroup(G, 0)",
        "StartTaskInGroup(G)",
        "StartTaskInGroup(1, Work)",
        "StartTaskInGroup(G, 1)",
        "StartSupervisedTask(G, Work, true, 0)",
        "StartSupervisedTask(G, Work, 1, 'bad')",
        "StartSupervisedTask(G, Work, 1)",
        "StartSupervisedTask(G, procedure() begin end, 1, 0)",
        "StartTaskInGroup(G, procedure() begin end)",
        "StartTaskInGroup(G, procedure(T: integer) begin end)",
        "StartTaskInGroup(G, procedure(T: CancellationSource) begin end)",
        "StartTaskInGroup(G, procedure(T: CancellationToken; Extra: integer) begin end)",
        "StartTaskInGroup(G, function(T: CancellationToken): result of integer, integer begin return Error(1) end)",
    ] {
        let source = format!(
            "program T; uses Std.Task; procedure Work(Token: CancellationToken); begin end; begin var G: TaskGroup := CreateTaskGroup(); {call} end."
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
uses Std.Task;
begin
  mutable var Count: integer := 0;
  var G: TaskGroup := CreateTaskGroup();
  StartTaskInGroup(G, procedure(Token: CancellationToken) begin Count := Count + 1 end)
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
uses Std.Task;
procedure NoValue(Token: CancellationToken); begin end;
function Number(Token: CancellationToken): integer; begin return 42 end;
function Outcome(Token: CancellationToken): result of integer, string; begin return Error('failed') end;
begin
  var G: TaskGroup := CreateTaskGroup();
  var A: task := StartTaskInGroup(G, NoValue);
  var B: task := StartTaskInGroup(G, Number);
  var C: task := StartTaskInGroup(G, Outcome);
  Wait(A);
  var N: integer := Wait(B);
  var R: result of integer, string := Wait(C);
  CloseTaskGroup(G)
end."#,
    );
}

#[test]
fn group_worker_does_not_erase_its_result_type() {
    let errors = check_errors(
        r#"program T;
uses Std.Task;
function Work(Token: CancellationToken): integer; begin return 7 end;
begin
  var G: TaskGroup := CreateTaskGroup();
  var Child: task := StartTaskInGroup(G, Work);
  var Wrong: string := Wait(Child)
end."#,
    );
    assert!(!errors.is_empty(), "worker result was erased");
}

#[test]
fn timed_group_close_preserves_result_and_failure_record_types() {
    check_ok(
        "program T; uses Std.Task, Std.Results; begin var G: TaskGroup := CreateTaskGroup(); var R: result of array of TaskFailure, string := CloseTaskGroupWithTimeout(G, 0); var Reports: array of TaskFailure := Unwrap(R) end.",
    );
    assert!(!check_errors("program T; uses Std.Task; begin var G: TaskGroup := CreateTaskGroup(); var Wrong: boolean := CloseTaskGroupWithTimeout(G, 0) end.").is_empty());
}

#[test]
fn completed_group_probe_preserves_optional_failure_report_type() {
    check_ok(
        "program T; uses Std.Task; begin var G: TaskGroup := CreateTaskGroup(); var R: option of array of TaskFailure := TryCloseCompletedTaskGroup(G) end.",
    );
    assert!(
        !check_errors(
            "program T; uses Std.Task; begin var G: TaskGroup := CreateTaskGroup(); var Wrong: boolean := TryCloseCompletedTaskGroup(G) end."
        )
        .is_empty()
    );
}
