use super::super::{check_errors, check_ok};
use fpas_diagnostics::codes::SEMA_TASK_BOUND_CALLABLE;
use fpas_parser::{ParseDiagnostic, parse};

#[test]
fn controlled_wait_any_checks_control_types_and_arity() {
    for call in [
        "WaitAnyWithTimeout(Tasks, 'bad')",
        "WaitAnyWithCancellation(Tasks, CreateCancellationSource())",
        "WaitAnyWithTimeout(Tasks)",
        "WaitAnyWithCancellation([1, 2], GetCancellationToken(CreateCancellationSource()))",
    ] {
        let source =
            format!("program T; uses Std.Task; begin var Tasks: array of task := []; {call} end.");
        assert!(!check_errors(&source).is_empty(), "{call}");
    }
}

#[test]
fn wait_any_rejects_non_task_arrays_and_reports_its_own_arity() {
    let errors = check_errors("program T; uses Std.Task; begin WaitAny([1, 2]) end.");
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("expected `array of task`"))
    );
    let errors = check_errors("program T; uses Std.Task; begin WaitAny() end.");
    assert!(errors.iter().any(|error| error.message.contains("WaitAny")));
}

#[test]
fn network_io_cancellation_requires_a_token_not_a_source() {
    let errors = check_errors(
        "\
program T;
uses Std.Net, Std.Task;
procedure Invalid(ConnectionValue: Std.Net.Connection);
begin
  Std.Net.ReadWithCancellation(ConnectionValue, 1, Std.Task.CreateCancellationSource());
  Std.Net.WriteWithCancellation(ConnectionValue, [1], Std.Task.CreateCancellationSource());
  Std.Net.ConnectWithCancellation('unused.invalid', 1, 1000, Std.Task.CreateCancellationSource());
  Std.Net.ConnectTlsWithCancellation('unused.invalid', 1, 1000, Std.Task.CreateCancellationSource())
end;
begin
end.",
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
        "\
program T;
uses Std.Task;

procedure LogAnswer();
begin
end;

begin
  var Tsk: task := go LogAnswer();
  Std.Task.Wait(Tsk)
end.",
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
        "\
program T;
uses Std.Task;

function Answer(): integer;
begin
  return 42
end;

begin
  var Tsk: task := go Answer();
  var Value: integer := Std.Task.Wait(Tsk)
end.",
    );
}

#[test]
fn task_wait_reports_assignment_mismatch() {
    let errors = check_errors(
        "\
program T;
uses Std.Task;

function Answer(): integer;
begin
  return 42
end;

begin
  var Tsk: task := go Answer();
  var Value: string := Std.Task.Wait(Tsk)
end.",
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
        "\
program T;
uses Std.Task;
begin
  mutable var Count: integer := 0;
  var Inc: procedure() :=
    procedure()
    begin
      Count := Count + 1
    end;
  go Inc()
end.",
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
        "\
program T;
uses Std.Task;
begin
  var Messages: channel of integer := CreateChannel(1);
  var Sent: result of boolean, string := Send(Messages, 42);
  var Received: result of integer, string := Receive(Messages);
  CloseChannel(Messages)
end.",
    );
}

#[test]
fn channel_send_rejects_the_wrong_element_type() {
    let errors = check_errors(
        "\
program T;
uses Std.Task;
begin
  var Messages: channel of integer := CreateChannel(1);
  Send(Messages, 'wrong')
end.",
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
        "\
program T;
uses Std.Task;
begin
  var Messages: channel of integer := CreateChannel(1);
  var Pending: result of option of integer, string := TryReceive(Messages);
  TrySend(Messages, 'wrong');
  SendWithTimeout(Messages, 1, 'soon');
  ReceiveWithTimeout(Messages, 'soon')
end.",
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
        "\
program T;
uses Std.Task;
begin
  mutable var Count: integer := 0;
  var Work: procedure() := procedure() begin Count := Count + 1 end;
  var Queue: channel of procedure() := CreateChannel(1);
  Send(Queue, Work)
end.",
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
        "\
program T;
uses Std.Task;
type WorkBox = record
  Work: procedure();
end;
begin
  mutable var Count: integer := 0;
  var Work: procedure() := procedure() begin Count := Count + 1 end;
  var ArrayQueue: channel of array of procedure() := CreateChannel(1);
  var RecordQueue: channel of WorkBox := CreateChannel(1);
  var ResultQueue: channel of result of procedure(), string := CreateChannel(1);
  var OptionQueue: channel of option of procedure() := CreateChannel(1);
  Send(ArrayQueue, [Work]);
  Send(RecordQueue, record Work := Work; end);
  Send(ResultQueue, Ok(Work));
  Send(OptionQueue, Some(Work))
end.",
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
        "\
program T;
uses Std.Task;
type WorkBox = record
  Work: procedure();
  Safe: integer;
end;
begin
  mutable var Count: integer := 0;
  var Work: procedure() := procedure() begin Count := Count + 1 end;
  var Boxed: WorkBox := record Work := Work; Safe := 7; end;
  var WorkQueue: channel of procedure() := CreateChannel(1);
  var SafeQueue: channel of integer := CreateChannel(1);
  Send(WorkQueue, Boxed.Work);
  Send(SafeQueue, Boxed.Safe)
end.",
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
