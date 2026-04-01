use super::{compile_and_run, compile_run_err};

#[test]
fn go_wait_supports_callable_values() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Task;

function AddFortyTwo(X: integer): integer;
begin
  return 40 + X
end;

begin
  var F: function(X: integer): integer := AddFortyTwo;
  var Tsk: task := go F(2);
  Std.Console.WriteLn(Std.Task.Wait(Tsk))
end.",
    );

    assert_eq!(out.lines, vec!["42"]);
}

#[test]
fn go_wait_all_supports_procedure_tasks() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Task;

procedure PrintValue(Value: integer);
begin
  Std.Console.WriteLn(Value)
end;

begin
  var T1: task := go PrintValue(10);
  var T2: task := go PrintValue(20);
  Std.Task.WaitAll([T1, T2])
end.",
    );

    let mut lines = out.lines;
    lines.sort();
    assert_eq!(lines, vec!["10", "20"]);
}

#[test]
fn wait_all_does_not_consume_task_results() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Task;

function Compute(Value: integer): integer;
begin
  return Value * 2
end;

begin
  var T1: task := go Compute(10);
  var T2: task := go Compute(11);
  Std.Task.WaitAll([T1, T2]);
  Std.Console.WriteLn(Std.Task.Wait(T1));
  Std.Console.WriteLn(Std.Task.Wait(T2))
end.",
    );

    assert_eq!(out.lines, vec!["20", "22"]);
}

#[test]
fn waiting_twice_on_the_same_task_reports_a_runtime_error() {
    let msg = compile_run_err(
        "\
program T;
uses Std.Console, Std.Task;

function Compute(): integer;
begin
  return 42
end;

begin
  var Tsk: task := go Compute();
  Std.Console.WriteLn(Std.Task.Wait(Tsk));
  Std.Console.WriteLn(Std.Task.Wait(Tsk))
end.",
    );

    assert!(
        msg.contains("was already awaited"),
        "expected already-awaited task error, got: {msg}"
    );
}

#[test]
fn go_supports_std_library_function_calls() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Task, Std.Conv;

begin
  var Tsk: task := go Std.Conv.IntToStr(12);
  Std.Console.WriteLn(Std.Task.Wait(Tsk))
end.",
    );

    assert_eq!(out.lines, vec!["12"]);
}

#[test]
fn go_supports_std_library_procedure_calls() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Task;

begin
  var Tsk: task := go Std.Console.WriteLn('async');
  Std.Task.Wait(Tsk);
  Std.Console.WriteLn('done')
end.",
    );

    assert_eq!(out.lines, vec!["async", "done"]);
}

#[test]
fn go_supports_record_method_calls() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Task, Std.Conv;

type Counter = record
  Value: integer;
  function Describe(Self: Counter): string;
  begin
    return 'count=' + Std.Conv.IntToStr(Self.Value)
  end;
end;

begin
  var C: Counter := record Value := 7; end;
  var Tsk: task := go C.Describe();
  Std.Console.WriteLn(Std.Task.Wait(Tsk))
end.",
    );

    assert_eq!(out.lines, vec!["count=7"]);
}
