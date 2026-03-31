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
uses Std.Console, Std.Task, Std.Channel;

procedure SendValue(Ch: channel of integer; Value: integer);
begin
  Std.Channel.Send(Ch, Value)
end;

begin
  var Ch: channel of integer := Std.Channel.MakeBuffered(2);
  var T1: task := go SendValue(Ch, 10);
  var T2: task := go SendValue(Ch, 20);
  Std.Task.WaitAll([T1, T2]);
  Std.Console.WriteLn(Std.Channel.Receive(Ch));
  Std.Console.WriteLn(Std.Channel.Receive(Ch))
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

#[test]
fn select_prefers_the_first_ready_arm() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Channel;

begin
  var Ch1: channel of integer := Std.Channel.MakeBuffered(1);
  var Ch2: channel of integer := Std.Channel.MakeBuffered(1);
  Std.Channel.Send(Ch1, 1);
  Std.Channel.Send(Ch2, 2);

  select
    case Value: integer from Ch1:
      Std.Console.WriteLn('first=' + Std.Conv.IntToStr(Value));
    case Other: integer from Ch2:
      Std.Console.WriteLn('second=' + Std.Conv.IntToStr(Other));
  end
end.",
    );

    assert_eq!(out.lines, vec!["first=1"]);
}

#[test]
fn select_with_default_runs_default_branch() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Channel;

begin
  var Ch: channel of integer := Std.Channel.Make();
  select
    case Value: integer from Ch:
      Std.Console.WriteLn(Value);
    default:
      Std.Console.WriteLn('idle');
  end
end.",
    );

    assert_eq!(out.lines, vec!["idle"]);
}

#[test]
fn select_without_default_waits_for_a_value() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Channel;

procedure Producer(Ch: channel of integer);
begin
  Std.Channel.Send(Ch, 99)
end;

begin
  var Ch: channel of integer := Std.Channel.Make();
  go Producer(Ch);

  select
    case Value: integer from Ch:
      Std.Console.WriteLn(Value);
  end
end.",
    );

    assert_eq!(out.lines, vec!["99"]);
}

#[test]
fn close_drains_buffer_and_try_receive_then_returns_none() {
    let out = compile_and_run(
        "\
program T;
uses Std.Console, Std.Channel, Std.Option;

begin
  var Ch: channel of integer := Std.Channel.MakeBuffered(2);
  Std.Channel.Send(Ch, 7);
  Std.Channel.Send(Ch, 8);
  Std.Channel.Close(Ch);

  Std.Console.WriteLn(Std.Channel.Receive(Ch));
  Std.Console.WriteLn(Std.Channel.Receive(Ch));
  Std.Console.WriteLn(Std.Option.IsNone(Std.Channel.TryReceive(Ch)))
end.",
    );

    assert_eq!(out.lines, vec!["7", "8", "true"]);
}

#[test]
fn sending_to_a_closed_channel_reports_a_runtime_error() {
    let msg = compile_run_err(
        "\
program T;
uses Std.Channel;

begin
  var Ch: channel of integer := Std.Channel.Make();
  Std.Channel.Close(Ch);
  Std.Channel.Send(Ch, 1)
end.",
    );

    assert!(
        msg.contains("Cannot send on a closed channel"),
        "expected closed-channel error, got: {msg}"
    );
}

#[test]
fn make_buffered_rejects_negative_capacity() {
    let msg = compile_run_err(
        "\
program T;
uses Std.Channel;

begin
  var Ch: channel of integer := Std.Channel.MakeBuffered(-1)
end.",
    );

    assert!(
        msg.contains("Channel buffer size must be a positive integer"),
        "expected positive-capacity error, got: {msg}"
    );
}
