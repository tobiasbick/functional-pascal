use super::super::{check_errors, check_ok};
use fpas_diagnostics::codes::SEMA_TASK_BOUND_CALLABLE;
use fpas_parser::{ParseDiagnostic, parse};

mod groups;

#[test]
fn selection_send_rejects_task_bound_values() {
    let errors = check_errors(
        r#"program T;
uses Std.Tasks as Tasks;
begin
  mutable var Count: integer := 0;
  var Work: procedure() := procedure() begin Count := Count + 1; end procedure;
  var Q: channel of procedure() := Tasks.CreateChannel(1);
  Tasks.SendCase(Q, Work, procedure(R: result of boolean, string) begin null; end procedure);
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
fn selection_cases_reject_wrong_payload_callback_and_control_types() {
    for call in [
        "Tasks.ReceiveCase(Q, procedure(R: result of string, string) begin null; end procedure)",
        "Tasks.SendCase(Q, 'wrong', procedure(R: result of boolean, string) begin null; end procedure)",
        "Tasks.SendCase(Q, 1, procedure(R: result of integer, string) begin null; end procedure)",
        "Tasks.TimerCase(0, function(): integer begin return 1; end function)",
        "Tasks.TimerCase('wrong', procedure() begin null; end procedure)",
        "Tasks.TaskCase(1, procedure() begin null; end procedure)",
        "Tasks.CancellationCase(Tasks.CreateCancellationSource(), procedure() begin null; end procedure)",
        "Tasks.Select([1])",
        "Tasks.CloseWaitCase(Tasks.CreateCancellationSource())",
    ] {
        let source = format!(
            "program T; uses Std.Tasks as Tasks; begin var Q: channel of integer := Tasks.CreateChannel(1); {call}; end program;"
        );
        assert!(
            !check_errors(&source).is_empty(),
            "accepted invalid selection: {call}"
        );
    }
}

#[test]
fn controlled_wait_any_checks_control_types_and_arity() {
    for call in [
        "Workers.WaitAnyWithTimeout(Handles, 'bad')",
        "Workers.WaitAnyWithCancellation(Handles, Workers.CreateCancellationSource())",
        "Workers.WaitAnyWithTimeout(Handles)",
        "Workers.WaitAnyWithCancellation([1, 2], Workers.GetCancellationToken(Workers.CreateCancellationSource()))",
    ] {
        let source = format!(
            "program T; uses Std.Tasks as Workers; begin var Handles: array of task := []; {call}; end program;"
        );
        assert!(!check_errors(&source).is_empty(), "{call}");
    }
}

#[test]
fn wait_any_rejects_non_task_arrays_and_reports_its_own_arity() {
    let errors = check_errors(
        r#"program T;  uses Std.Tasks as Tasks; begin Tasks.WaitAny([1, 2]); end program;"#,
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("expected `array of task`"))
    );
    let errors =
        check_errors(r#"program T;  uses Std.Tasks as Tasks; begin Tasks.WaitAny(); end program;"#);
    assert!(errors.iter().any(|error| error.message.contains("WaitAny")));
}

#[test]
fn network_io_cancellation_requires_a_token_not_a_source() {
    let errors = check_errors(
        r#"program T;
uses Std.Net as Net; uses Std.Tasks as Tasks;
procedure Invalid(ConnectionValue: Net.Connection);
begin Net.ReceiveBytesWithCancellation(ConnectionValue, 1, Tasks.CreateCancellationSource());
  Net.SendBytesWithCancellation(ConnectionValue, [1], Tasks.CreateCancellationSource());
  Net.ConnectWithCancellation('unused.invalid', 1, 1000, Tasks.CreateCancellationSource());
  Net.ConnectTlsWithCancellation('unused.invalid', 1, 1000, Tasks.CreateCancellationSource());
end procedure;
begin null; end program;"#,
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.message.contains("Type mismatch"))
            .count(),
        4,
        "errors: {errors:#?}"
    );
}

#[test]
fn go_accepts_procedure_calls_as_tasks() {
    check_ok(
        r#"program T;
uses Std.Tasks as Tasks;

procedure LogAnswer();
begin null;
end procedure;

begin
  var Tsk: task := go LogAnswer();
  Tasks.Wait(Tsk);
end program;"#,
    );
}

#[test]
fn go_requires_a_call_expression() {
    let (_, errors) = parse(
        "\
program T;
begin
  var Tsk: task := go 1
end.",
    );

    assert!(
        errors.iter().any(|error| match error {
            ParseDiagnostic::Parser(diagnostic) => diagnostic
                .message
                .contains("`go` requires a function or procedure call"),
            ParseDiagnostic::Lexer(_) => false,
        }),
        "errors: {errors:#?}"
    );
}

#[test]
fn task_wait_uses_task_result_type() {
    check_ok(
        r#"program T;
uses Std.Tasks as Tasks;

function Answer(): integer;
begin
  return 42;
end function;

begin
  var Tsk: task := go Answer();
  var Value: integer := Tasks.Wait(Tsk);
end program;"#,
    );
}

#[test]
fn task_wait_reports_assignment_mismatch() {
    let errors = check_errors(
        r#"program T;
uses Std.Tasks as Tasks;

function Answer(): integer;
begin
  return 42;
end function;

begin
  var Tsk: task := go Answer();
  var Value: string := Tasks.Wait(Tsk);
end program;"#,
    );

    assert!(
        errors.iter().any(|error| error
            .message
            .contains("Type mismatch in variable initializer")),
        "errors: {errors:#?}"
    );
}

#[test]
fn go_rejects_task_bound_mutable_closure() {
    let errors = check_errors(
        r#"program T;
uses Std.Tasks as Tasks;
begin
  mutable var Count: integer := 0;
  var Inc: procedure() :=
    procedure()
    begin
      Count := Count + 1;
    end procedure;
  go Inc();
end program;"#,
    );

    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("task-bound")
                || error.message.contains("Task-bound")),
        "errors: {errors:#?}"
    );
}

#[test]
fn typed_channel_operations_preserve_the_element_type() {
    check_ok(
        r#"program T;
uses Std.Tasks as Tasks;
begin
  var Messages: channel of integer := Tasks.CreateChannel(1);
  var Sent: result of boolean, string := Tasks.Send(Messages, 42);
  var Received: result of integer, string := Tasks.Receive(Messages);
  Tasks.CloseChannel(Messages);
end program;"#,
    );
}

#[test]
fn channel_send_rejects_the_wrong_element_type() {
    let errors = check_errors(
        r#"program T;
uses Std.Tasks as Tasks;
begin
  var Messages: channel of integer := Tasks.CreateChannel(1);
  Tasks.Send(Messages, 'wrong');
end program;"#,
    );

    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("Type mismatch in channel send")),
        "errors: {errors:#?}"
    );
}

#[test]
fn channel_wait_modes_preserve_element_and_timeout_types() {
    let errors = check_errors(
        r#"program T;
uses Std.Tasks as Tasks;
begin
  var Messages: channel of integer := Tasks.CreateChannel(1);
  var Pending: result of option of integer, string := Tasks.TryReceive(Messages);
  Tasks.TrySend(Messages, 'wrong');
  Tasks.SendWithTimeout(Messages, 1, 'soon');
  Tasks.ReceiveWithTimeout(Messages, 'soon');
end program;"#,
    );

    assert_eq!(
        errors
            .iter()
            .filter(|error| error.message.contains("Type mismatch"))
            .count(),
        3,
        "errors: {errors:#?}"
    );
}

#[test]
fn channel_send_rejects_task_bound_values() {
    let errors = check_errors(
        r#"program T;
uses Std.Tasks as Tasks;
begin
  mutable var Count: integer := 0;
  var Work: procedure() := procedure() begin Count := Count + 1; end procedure;
  var Queue: channel of procedure() := Tasks.CreateChannel(1);
  Tasks.Send(Queue, Work);
end program;"#,
    );

    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_TASK_BOUND_CALLABLE),
        "errors: {errors:#?}"
    );
}

#[test]
fn channel_send_rejects_task_bound_values_wrapped_in_aggregates() {
    let errors = check_errors(
        r#"program T;
uses Std.Tasks as Tasks;
 type WorkBox = record
  Work: procedure();
end record;
begin
  mutable var Count: integer := 0;
  var Work: procedure() := procedure() begin Count := Count + 1; end procedure;
  var ArrayQueue: channel of array of procedure() := Tasks.CreateChannel(1);
  var RecordQueue: channel of WorkBox := Tasks.CreateChannel(1);
  var ResultQueue: channel of result of procedure(), string := Tasks.CreateChannel(1);
  var OptionQueue: channel of option of procedure() := Tasks.CreateChannel(1);
  Tasks.Send(ArrayQueue, [Work]);
  Tasks.Send(RecordQueue, record Work := Work; end record);
  Tasks.Send(ResultQueue, Ok(Work));
  Tasks.Send(OptionQueue, Some(Work));
end program;"#,
    );

    assert_eq!(
        errors
            .iter()
            .filter(|error| error.code == SEMA_TASK_BOUND_CALLABLE)
            .count(),
        4,
        "errors: {errors:#?}"
    );
}

#[test]
fn channel_send_tracks_task_bound_postfix_results_by_selected_type() {
    let errors = check_errors(
        r#"program T;
uses Std.Tasks as Tasks;
 type WorkBox = record
  Work: procedure();
  Safe: integer;
end record;
begin
  mutable var Count: integer := 0;
  var Work: procedure() := procedure() begin Count := Count + 1; end procedure;
  var Boxed: WorkBox := record Work := Work; Safe := 7; end record;
  var WorkQueue: channel of procedure() := Tasks.CreateChannel(1);
  var SafeQueue: channel of integer := Tasks.CreateChannel(1);
  Tasks.Send(WorkQueue, Boxed.Work);
  Tasks.Send(SafeQueue, Boxed.Safe);
end program;"#,
    );

    assert_eq!(
        errors
            .iter()
            .filter(|error| error.code == SEMA_TASK_BOUND_CALLABLE)
            .count(),
        1,
        "errors: {errors:#?}"
    );
}

#[test]
fn typed_task_parameters_wait_for_their_declared_result_type() {
    check_ok(
        r#"program T;
uses Std.Tasks as Tasks;
function Seven(): integer;
begin
  return 7;
end function;
function Doubled(Job: task of integer): integer;
begin
  return Tasks.Wait(Job) * 2;
end function;
function First(Jobs: array of task of integer): integer;
begin
  return Tasks.Wait(Jobs[Tasks.WaitAny(Jobs)]);
end function;
begin
  var Job: task of integer := go Seven();
  var Inferred: task := go Seven();
  var Total: integer := Doubled(Job) + First([Inferred]);
end program;"#,
    );
}

#[test]
fn typed_tasks_reject_a_different_result_type() {
    for source in [
        r#"program T;  uses Std.Tasks as Tasks; function Seven(): integer; begin return 7; end function; begin var Job: task of string := go Seven(); end program;"#,
        r#"program T;  uses Std.Tasks as Tasks; function Seven(): integer; begin return 7; end function; function Name(Job: task of string): string; begin return Tasks.Wait(Job); end function; begin var Job: task := go Seven(); var Text: string := Name(Job); end program;"#,
        r#"program T;  uses Std.Tasks as Tasks; function Count(Job: task of integer): string; begin return Tasks.Wait(Job); end function; begin null; end program;"#,
    ] {
        assert!(!check_errors(source).is_empty(), "{source}");
    }
}
