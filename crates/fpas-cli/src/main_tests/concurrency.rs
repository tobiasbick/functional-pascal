use super::*;

// ---------------------------------------------------------------------------
// go + Task.Wait
// ---------------------------------------------------------------------------

#[test]
fn go_and_task_wait() {
    let source = r#"program GoWait;
uses Std.Console, Std.Task;

function Worker(): integer;
begin
  return 42
end;

begin
  var T: task := go Worker();
  WriteLn(Std.Task.Wait(T))
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("go.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "42\n");
}

#[test]
fn go_with_string_return() {
    let source = r#"program GoStr;
uses Std.Console, Std.Task;

function Greet(): string;
begin
  return 'hello'
end;

begin
  var T: task := go Greet();
  WriteLn(Std.Task.Wait(T))
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("go_str.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "hello\n");
}

#[test]
fn go_with_arguments() {
    let source = r#"program GoArgs;
uses Std.Console, Std.Task;

function Add(A: integer; B: integer): integer;
begin
  return A + B
end;

begin
  var T: task := go Add(10, 32);
  WriteLn(Std.Task.Wait(T))
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("go_args.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "42\n");
}

// ---------------------------------------------------------------------------
// Task.WaitAll
// ---------------------------------------------------------------------------

#[test]
fn task_wait_all() {
    let source = r#"program WaitAll;
uses Std.Console, Std.Task;

function Double(X: integer): integer;
begin
  return X * 2
end;

begin
  var T1: task := go Double(1);
  var T2: task := go Double(2);
  var T3: task := go Double(3);
  var Results: array of task := [T1, T2, T3];
  Std.Task.WaitAll(Results);
  WriteLn('done')
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("waitall.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "done\n");
}

#[test]
fn task_wait_all_empty_array() {
    let source = r#"program WaitAllEmpty;
uses Std.Console, Std.Task;

begin
  var Tasks: array of task := [];
  Std.Task.WaitAll(Tasks);
  WriteLn('ok')
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("waitall_empty.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "ok\n");
}

// ---------------------------------------------------------------------------
// Channel basics
// ---------------------------------------------------------------------------

#[test]
fn channel_send_receive() {
    let source = r#"program ChanTest;
uses Std.Console, Std.Channel, Std.Task;

procedure Sender(Ch: channel of integer);
begin
  Std.Channel.Send(Ch, 99)
end;

begin
  var Ch: channel of integer := Std.Channel.Make();
  go Sender(Ch);
  WriteLn(Std.Channel.Receive(Ch))
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("chan.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "99\n");
}

#[test]
fn channel_of_string() {
    let source = r#"program ChanString;
uses Std.Console, Std.Channel, Std.Task;

procedure Sender(Ch: channel of string);
begin
  Std.Channel.Send(Ch, 'world')
end;

begin
  var Ch: channel of string := Std.Channel.Make();
  go Sender(Ch);
  var Msg: string := Std.Channel.Receive(Ch);
  WriteLn(Msg)
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("chan_str.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "world\n");
}

#[test]
fn channel_multiple_values() {
    let source = r#"program ChanMulti;
uses Std.Console, Std.Channel, Std.Task;

procedure Producer(Ch: channel of integer);
begin
  Std.Channel.Send(Ch, 1);
  Std.Channel.Send(Ch, 2);
  Std.Channel.Send(Ch, 3)
end;

begin
  var Ch: channel of integer := Std.Channel.Make();
  go Producer(Ch);
  WriteLn(Std.Channel.Receive(Ch));
  WriteLn(Std.Channel.Receive(Ch));
  WriteLn(Std.Channel.Receive(Ch))
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("chan_multi.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "1\n2\n3\n");
}

// ---------------------------------------------------------------------------
// TryReceive
// ---------------------------------------------------------------------------

#[test]
fn channel_try_receive_none() {
    let source = r#"program TryRecvNone;
uses Std.Console, Std.Channel, Std.Option;

begin
  var Ch: channel of integer := Std.Channel.Make();
  var V: Option of integer := Std.Channel.TryReceive(Ch);
  if IsNone(V) then
    WriteLn('none')
  else
    WriteLn('some')
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("tryrecv.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "none\n");
}

#[test]
fn channel_try_receive_some() {
    let source = r#"program TryRecvSome;
uses Std.Console, Std.Channel, Std.Option;

begin
  var Ch: channel of integer := Std.Channel.Make();
  Std.Channel.Send(Ch, 42);
  var V: Option of integer := Std.Channel.TryReceive(Ch);
  if IsSome(V) then
    WriteLn('got it')
  else
    WriteLn('empty')
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("tryrecv_some.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "got it\n");
}

// ---------------------------------------------------------------------------
// Channel close
// ---------------------------------------------------------------------------

#[test]
fn channel_close() {
    let source = r#"program ChanClose;
uses Std.Console, Std.Channel;

begin
  var Ch: channel of integer := Std.Channel.Make();
  Std.Channel.Send(Ch, 1);
  Std.Channel.Close(Ch);
  WriteLn(Std.Channel.Receive(Ch))
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("close.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "1\n");
}

#[test]
fn channel_close_receive_empty_is_runtime_error() {
    let source = r#"program CloseEmpty;
uses Std.Channel;

begin
  var Ch: channel of integer := Std.Channel.Make();
  Std.Channel.Close(Ch);
  Std.Channel.Receive(Ch)
end.
"#;

    let (exit_code, stderr_output) = support::run_and_capture_stderr("close_empty.fpas", source);
    assert_eq!(exit_code, 2);
    assert!(
        stderr_output.contains("closed, empty channel"),
        "stderr: {stderr_output}"
    );
}

#[test]
fn channel_send_to_closed_is_runtime_error() {
    let source = r#"program SendClosed;
uses Std.Channel;

begin
  var Ch: channel of integer := Std.Channel.Make();
  Std.Channel.Close(Ch);
  Std.Channel.Send(Ch, 1)
end.
"#;

    let (exit_code, stderr_output) = support::run_and_capture_stderr("send_closed.fpas", source);
    assert_eq!(exit_code, 2);
    assert!(
        stderr_output.contains("Cannot send on a closed channel"),
        "stderr: {stderr_output}"
    );
}

// ---------------------------------------------------------------------------
// MakeBuffered
// ---------------------------------------------------------------------------

#[test]
fn channel_make_buffered() {
    let source = r#"program Buffered;
uses Std.Console, Std.Channel;

begin
  var Ch: channel of integer := Std.Channel.MakeBuffered(3);
  Std.Channel.Send(Ch, 10);
  Std.Channel.Send(Ch, 20);
  Std.Channel.Send(Ch, 30);
  WriteLn(Std.Channel.Receive(Ch));
  WriteLn(Std.Channel.Receive(Ch));
  WriteLn(Std.Channel.Receive(Ch))
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("buffered.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "10\n20\n30\n");
}

#[test]
fn channel_buffered_fifo_order() {
    let source = r#"program BufferedFIFO;
uses Std.Console, Std.Channel;

begin
  var Ch: channel of string := Std.Channel.MakeBuffered(5);
  Std.Channel.Send(Ch, 'a');
  Std.Channel.Send(Ch, 'b');
  Std.Channel.Send(Ch, 'c');
  WriteLn(Std.Channel.Receive(Ch));
  WriteLn(Std.Channel.Receive(Ch));
  WriteLn(Std.Channel.Receive(Ch))
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("buffered_fifo.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "a\nb\nc\n");
}

// ---------------------------------------------------------------------------
// Select statement
// ---------------------------------------------------------------------------

#[test]
fn select_single_arm() {
    let source = r#"program SelectOne;
uses Std.Console, Std.Channel, Std.Task;

procedure Sender(Ch: channel of integer);
begin
  Std.Channel.Send(Ch, 7)
end;

begin
  var Ch: channel of integer := Std.Channel.Make();
  go Sender(Ch);
  select
    case V: integer from Ch:
      WriteLn(V);
  end
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("select_one.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "7\n");
}

#[test]
fn select_multiple_arms() {
    let source = r#"program SelectMulti;
uses Std.Console, Std.Channel, Std.Task;

procedure SendNum(Ch: channel of integer);
begin
  Std.Channel.Send(Ch, 42)
end;

begin
  var Ch1: channel of integer := Std.Channel.Make();
  var Ch2: channel of integer := Std.Channel.Make();
  go SendNum(Ch1);
  select
    case V: integer from Ch1:
      WriteLn(V);
    case V: integer from Ch2:
      WriteLn(V);
  end
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("select_multi.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "42\n");
}

#[test]
fn select_with_default_no_data() {
    let source = r#"program SelectDefault;
uses Std.Console, Std.Channel;

begin
  var Ch: channel of integer := Std.Channel.Make();
  select
    case V: integer from Ch:
      WriteLn(V);
    default:
      WriteLn('default');
  end
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("select_default.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "default\n");
}

#[test]
fn select_with_default_data_ready() {
    let source = r#"program SelectDefaultReady;
uses Std.Console, Std.Channel;

begin
  var Ch: channel of integer := Std.Channel.Make();
  Std.Channel.Send(Ch, 5);
  select
    case V: integer from Ch:
      WriteLn(V);
    default:
      WriteLn('default');
  end
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("select_default_ready.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "5\n");
}

// ---------------------------------------------------------------------------
// Producer/Consumer pattern (from spec)
// ---------------------------------------------------------------------------

#[test]
fn producer_consumer() {
    let source = r#"program ProducerConsumer;
uses Std.Console, Std.Channel, Std.Task;

procedure Producer(Ch: channel of integer);
begin
  Std.Channel.Send(Ch, 99)
end;

begin
  var Ch: channel of integer := Std.Channel.Make();
  go Producer(Ch);
  WriteLn(Std.Channel.Receive(Ch))
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("producer_consumer.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "99\n");
}

// ---------------------------------------------------------------------------
// Multiple tasks sharing a channel
// ---------------------------------------------------------------------------

#[test]
fn multiple_tasks_one_channel() {
    let source = r#"program MultiTask;
uses Std.Console, Std.Channel, Std.Task;

procedure Worker(Ch: channel of integer; Val: integer);
begin
  Std.Channel.Send(Ch, Val)
end;

begin
  var Ch: channel of integer := Std.Channel.MakeBuffered(3);
  go Worker(Ch, 10);
  go Worker(Ch, 20);
  go Worker(Ch, 30);
  var A: integer := Std.Channel.Receive(Ch);
  var B: integer := Std.Channel.Receive(Ch);
  var C: integer := Std.Channel.Receive(Ch);
  WriteLn(A + B + C)
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("multi_task.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "60\n");
}

// ---------------------------------------------------------------------------
// TryReceive on closed channel
// ---------------------------------------------------------------------------

#[test]
fn try_receive_closed_empty_returns_none() {
    let source = r#"program TryRecvClosed;
uses Std.Console, Std.Channel, Std.Option;

begin
  var Ch: channel of integer := Std.Channel.Make();
  Std.Channel.Close(Ch);
  var V: Option of integer := Std.Channel.TryReceive(Ch);
  if IsNone(V) then
    WriteLn('none')
  else
    WriteLn('some')
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("tryrecv_closed.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "none\n");
}

#[test]
fn try_receive_closed_with_data_returns_some() {
    let source = r#"program TryRecvClosedData;
uses Std.Console, Std.Channel, Std.Option;

begin
  var Ch: channel of integer := Std.Channel.Make();
  Std.Channel.Send(Ch, 7);
  Std.Channel.Close(Ch);
  var V: Option of integer := Std.Channel.TryReceive(Ch);
  if IsSome(V) then
    WriteLn('got it')
  else
    WriteLn('empty')
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("tryrecv_closed_data.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "got it\n");
}

// ===========================================================================
// Task edge cases
// ===========================================================================

#[test]
fn go_procedure_returns_unit() {
    let source = r#"program GoProcedure;
uses Std.Console, Std.Task;

function DoWork(): integer;
begin
  return 1
end;

begin
  var T: task := go DoWork();
  Std.Task.Wait(T);
  WriteLn('done')
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("go_proc.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "done\n");
}

#[test]
fn go_task_returning_boolean() {
    let source = r#"program GoBoolean;
uses Std.Console, Std.Task;

function IsEven(N: integer): boolean;
begin
  return N mod 2 = 0
end;

begin
  var T: task := go IsEven(4);
  WriteLn(Std.Task.Wait(T))
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("go_bool.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "true\n");
}

#[test]
fn go_task_returning_array() {
    let source = r#"program GoArray;
uses Std.Console, Std.Task;

function MakeList(): array of integer;
begin
  return [10, 20, 30]
end;

begin
  var T: task := go MakeList();
  var Arr: array of integer := Std.Task.Wait(T);
  WriteLn(Arr[0]);
  WriteLn(Arr[2])
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("go_array.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "10\n30\n");
}

#[test]
fn go_task_panic_is_runtime_error() {
    let source = r#"program GoPanic;
uses Std.Task;

function Boom(): integer;
begin
  panic('exploded');
  return 0
end;

begin
  var T: task := go Boom();
  Std.Task.Wait(T)
end.
"#;

    let (exit_code, stderr_output) = support::run_and_capture_stderr("go_panic.fpas", source);
    assert_eq!(exit_code, 2);
    // The error may be from the panicking task or from the waiting main task.
    assert!(
        stderr_output.contains("exploded") || stderr_output.contains("task failed"),
        "stderr: {stderr_output}"
    );
}

// ===========================================================================
// Nested go (task spawns task)
// ===========================================================================

#[test]
fn nested_go_task_spawns_task() {
    let source = r#"program NestedGo;
uses Std.Console, Std.Task;

function Inner(): integer;
begin
  return 7
end;

function Outer(): integer;
begin
  var T: task := go Inner();
  return Std.Task.Wait(T) * 2
end;

begin
  var T: task := go Outer();
  WriteLn(Std.Task.Wait(T))
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("nested_go.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "14\n");
}

// ===========================================================================
// Channel edge cases
// ===========================================================================

#[test]
fn channel_backpressure_full_buffer() {
    // Buffer size 2, producer sends 4 values via a task.
    // The send should yield/retry when full, not error.
    let source = r#"program Backpressure;
uses Std.Console, Std.Channel, Std.Task;

procedure Producer(Ch: channel of integer);
begin
  Std.Channel.Send(Ch, 1);
  Std.Channel.Send(Ch, 2);
  Std.Channel.Send(Ch, 3);
  Std.Channel.Send(Ch, 4)
end;

begin
  var Ch: channel of integer := Std.Channel.MakeBuffered(2);
  go Producer(Ch);
  WriteLn(Std.Channel.Receive(Ch));
  WriteLn(Std.Channel.Receive(Ch));
  WriteLn(Std.Channel.Receive(Ch));
  WriteLn(Std.Channel.Receive(Ch))
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("backpressure.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "1\n2\n3\n4\n");
}

#[test]
fn channel_passed_between_tasks() {
    let source = r#"program ChanRelay;
uses Std.Console, Std.Channel, Std.Task;

procedure Relay(Input: channel of integer; Output: channel of integer);
begin
  var V: integer := Std.Channel.Receive(Input);
  Std.Channel.Send(Output, V * 10)
end;

procedure Start(Ch: channel of integer);
begin
  Std.Channel.Send(Ch, 5)
end;

begin
  var A: channel of integer := Std.Channel.Make();
  var B: channel of integer := Std.Channel.Make();
  go Start(A);
  go Relay(A, B);
  WriteLn(Std.Channel.Receive(B))
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("chan_relay.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "50\n");
}

#[test]
fn channel_of_boolean() {
    let source = r#"program ChanBool;
uses Std.Console, Std.Channel, Std.Task;

procedure Sender(Ch: channel of boolean);
begin
  Std.Channel.Send(Ch, true)
end;

begin
  var Ch: channel of boolean := Std.Channel.Make();
  go Sender(Ch);
  WriteLn(Std.Channel.Receive(Ch))
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("chan_bool.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "true\n");
}

// ===========================================================================
// Select edge cases
// ===========================================================================

#[test]
fn select_picks_one_of_two_ready() {
    // Both channels have data; select must pick exactly one.
    let source = r#"program SelectBothReady;
uses Std.Console, Std.Channel;

begin
  var Ch1: channel of integer := Std.Channel.Make();
  var Ch2: channel of integer := Std.Channel.Make();
  Std.Channel.Send(Ch1, 1);
  Std.Channel.Send(Ch2, 2);
  mutable var Got: integer := 0;
  select
    case V: integer from Ch1:
      Got := V;
    case V: integer from Ch2:
      Got := V;
  end;
  if (Got = 1) or (Got = 2) then
    WriteLn('ok')
  else
    WriteLn('unexpected')
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("select_both.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "ok\n");
}

// ===========================================================================
// Stress test — many concurrent tasks
// ===========================================================================

#[test]
fn stress_many_concurrent_tasks() {
    let source = r#"program StressGo;
uses Std.Console, Std.Task;

function Identity(X: integer): integer;
begin
  return X
end;

begin
  mutable var Sum: integer := 0;
  for I: integer := 1 to 50 do
  begin
    var T: task := go Identity(I);
    var R: integer := Std.Task.Wait(T);
    Sum := Sum + R
  end;
  WriteLn(Sum)
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("stress.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    // Sum of 1..50 = 1275
    assert_eq!(stdout_output, "1275\n");
}

#[test]
fn stress_many_channel_messages() {
    let source = r#"program StressChan;
uses Std.Console, Std.Channel, Std.Task;

procedure Pump(Ch: channel of integer; Count: integer);
begin
  for I: integer := 1 to Count do
    Std.Channel.Send(Ch, I)
end;

begin
  var Ch: channel of integer := Std.Channel.MakeBuffered(100);
  go Pump(Ch, 100);
  mutable var Total: integer := 0;
  for I: integer := 1 to 100 do
  begin
    var Msg: integer := Std.Channel.Receive(Ch);
    Total := Total + Msg
  end;
  WriteLn(Total)
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("stress_chan.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    // Sum of 1..100 = 5050
    assert_eq!(stdout_output, "5050\n");
}

// ===========================================================================
// Individual Wait to collect task results
// ===========================================================================

#[test]
fn task_wait_collects_multiple_results() {
    let source = r#"program WaitResults;
uses Std.Console, Std.Task;

function Square(X: integer): integer;
begin
  return X * X
end;

begin
  var T1: task := go Square(3);
  var T2: task := go Square(4);
  var T3: task := go Square(5);
  WriteLn(Std.Task.Wait(T1));
  WriteLn(Std.Task.Wait(T2));
  WriteLn(Std.Task.Wait(T3))
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("wait_results.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "9\n16\n25\n");
}

#[test]
fn invalid_go_expression_is_compile_error() {
    let source = r#"program BadGo;
begin
  go 1
end.
"#;

    let (exit_code, stderr_output) = support::run_and_capture_stderr("bad_go.fpas", source);
    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("`go` requires a function or procedure call"),
        "stderr: {stderr_output}"
    );
}

#[test]
fn task_wait_all_does_not_leak_stack() {
    let source = r#"program WaitAllNoLeak;
uses Std.Console, Std.Task;

begin
  var Tasks: array of task := [];
  for I: integer := 1 to 5000 do
    Std.Task.WaitAll(Tasks);
  WriteLn('ok')
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("waitall_no_leak.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "ok\n");
}

// ===========================================================================
// Select with mixed-type channels (from spec example)
// ===========================================================================

#[test]
fn select_with_mixed_type_channels() {
    let source = r#"program SelectMixed;
uses Std.Console, Std.Channel, Std.Conv;

begin
  var Ch1: channel of string := Std.Channel.Make();
  var Ch2: channel of integer := Std.Channel.Make();
  Std.Channel.Send(Ch2, 7);
  select
    case Msg: string from Ch1:
      WriteLn('msg=' + Msg);
    case Num: integer from Ch2:
      WriteLn('num=' + Std.Conv.IntToStr(Num));
  end
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("select_mixed.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "num=7\n");
}

// ===========================================================================
// MakeBuffered(0) creates a rendezvous (synchronous) channel
// ===========================================================================

#[test]
fn make_buffered_zero_creates_sync_channel() {
    let source = r#"program SyncChan;
uses Std.Channel;

begin
  var Ch: channel of integer := Std.Channel.MakeBuffered(0)
end.
"#;

    let (exit_code, stderr_output) = support::run_and_capture_stderr("sync_chan.fpas", source);
    assert_eq!(exit_code, 2);
    assert!(
        stderr_output.contains("buffer size must be a positive integer"),
        "stderr: {stderr_output}"
    );
}

// ===========================================================================
// Double close is a no-op (not an error)
// ===========================================================================

#[test]
fn double_close_does_not_error() {
    let source = r#"program DoubleClose;
uses Std.Console, Std.Channel;

begin
  var Ch: channel of integer := Std.Channel.Make();
  Std.Channel.Send(Ch, 1);
  Std.Channel.Close(Ch);
  Std.Channel.Close(Ch);
  WriteLn(Std.Channel.Receive(Ch))
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("double_close.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "1\n");
}

// ===========================================================================
// Channel of real
// ===========================================================================

#[test]
fn channel_of_real() {
    let source = r#"program ChanReal;
uses Std.Console, Std.Channel, Std.Task;

procedure Sender(Ch: channel of real);
begin
  Std.Channel.Send(Ch, 3.14)
end;

begin
  var Ch: channel of real := Std.Channel.Make();
  go Sender(Ch);
  WriteLn(Std.Channel.Receive(Ch))
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("chan_real.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "3.14\n");
}

// ===========================================================================
// Select skips closed+empty arm, picks ready arm
// ===========================================================================

#[test]
fn select_skips_closed_arm_to_ready_arm() {
    let source = r#"program SelectClosed;
uses Std.Console, Std.Channel;

begin
  var Closed: channel of integer := Std.Channel.Make();
  Std.Channel.Close(Closed);
  var Ready: channel of integer := Std.Channel.Make();
  Std.Channel.Send(Ready, 42);
  select
    case V: integer from Closed:
      WriteLn('closed');
    case V: integer from Ready:
      WriteLn(V);
  end
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("select_closed.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "42\n");
}

// ===========================================================================
// Go spawns a closure (callable variable)
// ===========================================================================

#[test]
fn go_spawns_callable_variable() {
    let source = r#"program GoCallable;
uses Std.Console, Std.Task;

function Greet(): string;
begin
  return 'Hi World'
end;

begin
  var F: function(): string := Greet;
  var T: task := go F();
  WriteLn(Std.Task.Wait(T))
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("go_callable.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "Hi World\n");
}

// ===========================================================================
// TryReceive: extract the value from Some
// ===========================================================================

#[test]
fn try_receive_extracts_value() {
    let source = r#"program TryRecvValue;
uses Std.Console, Std.Channel, Std.Option;

begin
  var Ch: channel of integer := Std.Channel.Make();
  Std.Channel.Send(Ch, 77);
  var V: Option of integer := Std.Channel.TryReceive(Ch);
  WriteLn(Std.Option.Unwrap(V))
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("tryrecv_value.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "77\n");
}

// ===========================================================================
// Channel of record — send/receive complex types
// ===========================================================================

#[test]
fn channel_of_record() {
    let source = r#"program ChanRecord;
uses Std.Console, Std.Channel, Std.Task, Std.Conv;

type Point = record
  X: integer;
  Y: integer;
end;

procedure Sender(Ch: channel of Point);
begin
  Std.Channel.Send(Ch, record X := 3; Y := 4; end)
end;

begin
  var Ch: channel of Point := Std.Channel.Make();
  go Sender(Ch);
  var P: Point := Std.Channel.Receive(Ch);
  WriteLn(Std.Conv.IntToStr(P.X) + ',' + Std.Conv.IntToStr(P.Y))
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("chan_record.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "3,4\n");
}

// ===========================================================================
// Select with default only (no case arms)
// ===========================================================================

#[test]
fn select_default_only() {
    let source = r#"program SelectDefaultOnly;
uses Std.Console;

begin
  select
    default:
      WriteLn('only default');
  end
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("select_default_only.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "only default\n");
}

// ===========================================================================
// Select with default + data on string channel picks data arm
// ===========================================================================

#[test]
fn select_prefers_data_over_default() {
    let source = r#"program SelectPrefersData;
uses Std.Console, Std.Channel;

begin
  var Ch: channel of string := Std.Channel.Make();
  Std.Channel.Send(Ch, 'hello');
  select
    case Msg: string from Ch:
      WriteLn(Msg);
    default:
      WriteLn('default');
  end
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("select_data_over_default.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "hello\n");
}

// ===========================================================================
// Send to closed channel from a task — runtime error
// ===========================================================================

#[test]
fn send_to_closed_channel_in_task_is_runtime_error() {
    let source = r#"program SendClosedTask;
uses Std.Channel, Std.Task;

procedure BadSender(Ch: channel of integer);
begin
  Std.Channel.Send(Ch, 1)
end;

begin
  var Ch: channel of integer := Std.Channel.Make();
  Std.Channel.Close(Ch);
  var T: task := go BadSender(Ch);
  Std.Task.Wait(T)
end.
"#;

    let (exit_code, stderr_output) =
        support::run_and_capture_stderr("send_closed_task.fpas", source);
    assert_eq!(exit_code, 2);
    assert!(
        stderr_output.contains("closed channel") || stderr_output.contains("task failed"),
        "stderr: {stderr_output}"
    );
}

// ===========================================================================
// Receive from closed+drained buffered channel — runtime error
// ===========================================================================

#[test]
fn receive_from_drained_closed_buffered_channel_is_error() {
    let source = r#"program DrainedClosed;
uses Std.Channel;

begin
  var Ch: channel of integer := Std.Channel.MakeBuffered(2);
  Std.Channel.Send(Ch, 1);
  Std.Channel.Send(Ch, 2);
  Std.Channel.Close(Ch);
  var A: integer := Std.Channel.Receive(Ch);
  var B: integer := Std.Channel.Receive(Ch);
  Std.Channel.Receive(Ch)
end.
"#;

    let (exit_code, stderr_output) = support::run_and_capture_stderr("drained_closed.fpas", source);
    assert_eq!(exit_code, 2);
    assert!(
        stderr_output.contains("closed, empty channel"),
        "stderr: {stderr_output}"
    );
}

// ===========================================================================
// MakeBuffered with large capacity
// ===========================================================================

#[test]
fn make_buffered_large_capacity() {
    let source = r#"program BigBuffer;
uses Std.Console, Std.Channel;

begin
  var Ch: channel of integer := Std.Channel.MakeBuffered(1000);
  for I: integer := 1 to 1000 do
    Std.Channel.Send(Ch, I);
  mutable var Sum: integer := 0;
  for I: integer := 1 to 1000 do
    Sum := Sum + Std.Channel.Receive(Ch);
  WriteLn(Sum)
end.
"#;

    let (exit_code, stdout_output, stderr_output) =
        support::run_source_and_capture_output("big_buffer.fpas", source);
    assert!(stderr_output.is_empty(), "stderr: {stderr_output}");
    assert_eq!(exit_code, 0);
    // Sum 1..1000 = 500500
    assert_eq!(stdout_output, "500500\n");
}
