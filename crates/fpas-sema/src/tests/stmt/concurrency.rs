use super::super::{check_errors, check_ok};
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
fn task_wait_all_requires_task_array() {
    let errors = check_errors(
        "\
program T;
uses Std.Task;
begin
  Std.Task.WaitAll([1, 2, 3])
end.",
    );

    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("Type mismatch in task list")),
        "errors: {errors:#?}"
    );
}
