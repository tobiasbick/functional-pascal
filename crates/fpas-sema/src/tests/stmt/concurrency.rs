use super::super::{check_errors, check_ok};

#[test]
fn channel_receive_uses_channel_element_type() {
    check_ok(
        "\
program T;
uses Std.Channel;
begin
  var Ch: channel of integer := Std.Channel.Make();
  var Value: integer := Std.Channel.Receive(Ch)
end.",
    );
}

#[test]
fn channel_receive_reports_assignment_mismatch() {
    let errors = check_errors(
        "\
program T;
uses Std.Channel;
begin
  var Ch: channel of integer := Std.Channel.Make();
  var Value: string := Std.Channel.Receive(Ch)
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
fn channel_send_reports_value_type_mismatch() {
    let errors = check_errors(
        "\
program T;
uses Std.Channel;
begin
  var Ch: channel of integer := Std.Channel.Make();
  Std.Channel.Send(Ch, 'oops')
end.",
    );

    assert!(
        errors.iter().any(|error| error
            .message
            .contains("Type mismatch in channel send value")),
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
    let errors = check_errors(
        "\
program T;
begin
  var Tsk: task := go 1
end.",
    );

    assert!(
        errors.iter().any(|error| error
            .message
            .contains("`go` requires a function or procedure call")),
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

#[test]
fn select_requires_channel_source() {
    let errors = check_errors(
        "\
program T;
begin
  select
    case Value: integer from 1:
      return;
  end
end.",
    );

    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("Type mismatch in select arm source")),
        "errors: {errors:#?}"
    );
}

#[test]
fn select_requires_binding_type_to_match_channel_element_type() {
    let errors = check_errors(
        "\
program T;
uses Std.Channel;
begin
  var Ch: channel of integer := Std.Channel.Make();
  select
    case Value: string from Ch:
      return;
  end
end.",
    );

    assert!(
        errors.iter().any(|error| error
            .message
            .contains("Type mismatch in select arm binding")),
        "errors: {errors:#?}"
    );
}
