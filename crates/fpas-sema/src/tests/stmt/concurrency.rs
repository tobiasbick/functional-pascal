use super::super::{check_errors, check_ok};
use fpas_diagnostics::codes::SEMA_TASK_BOUND_CALLABLE;
use fpas_parser::{ParseDiagnostic, parse};
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
