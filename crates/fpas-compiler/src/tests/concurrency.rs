use super::*;

mod network_connect;
mod network_read;
mod network_write;
mod task_results;
mod wait_any;

#[test]
fn task_spawn_with_arguments_keeps_loop_branch_addresses_aligned() {
    assert_succeeds(
        r#"program RegisterTaskArgumentLoop;
uses Std.Arrays as Arrays; uses Std.Tasks as Tasks2;
function Worker(Value: integer): integer;
begin
  return Value + 1;
end function;
begin
  mutable var Tasks: array of (task) := [];
  for Index: integer := 1 to 8 do
  begin
    Arrays.Push(Tasks, go Worker(Index));
  end; end for;
  Tasks2.WaitAll(Tasks);
  if Arrays.Length(Tasks) <> 8 then panic('task loop count mismatch'); end if;
end program;"#,
    );
}

#[test]
fn retained_task_spawn_and_wait_execute() {
    let execution = assert_succeeds(
        r#"program RegisterTasks;
uses Std.Console as Console; uses Std.Tasks as Tasks;

function Add(A: integer; B: integer): integer;
begin
  return A + B;
end function;

begin
  var T: task := go Add(20, 22);
  Console.WriteLn(Tasks.Wait(T));
end program;"#,
    );
    assert_eq!(execution.value, fpas_bytecode::Value::Unit);
}

#[test]
fn detached_task_executes_on_register_pool() {
    assert_succeeds(
        r#"program RegisterDetached;
uses Std.Console as Console;

procedure Work();
begin
  Console.WriteLn('worker');
end procedure;

begin
  go Work();
end program;"#,
    );
}

#[test]
fn timeslice_preserves_nested_frames_and_live_aggregate_registers() {
    assert_succeeds(
        r#"program RegisterTaskFrames;
uses Std.Tasks as Tasks;

function Burn(Count: integer): integer;
begin
  mutable var I: integer := 0;
  while I < Count do
    I := I + 1; end while;
  return I;
end function;

function Work(): integer;
begin
  var Values: array of (integer) := [40, 2];
  return Burn(700) - 700 + Values[0] + Values[1];
end function;

begin
  var T: task := go Work();
  if Tasks.Wait(T) <> 42 then panic('task state was not restored'); end if;
end program;"#,
    );
}

#[test]
fn cooperative_sleep_releases_register_pool_worker() {
    assert_succeeds(
        r#"program RegisterTaskSleep;
uses Std.Tasks as Tasks; uses Std.Time as Time;

function Work(Value: integer): integer;
begin
  Time.Sleep(1);
  return Value;
end function;

begin
  var A: task := go Work(42);
  discard Tasks.Wait(A);
end program;"#,
    );
}

#[test]
fn cancellation_token_interrupts_network_accept_end_to_end() {
    let reservation = std::net::TcpListener::bind("127.0.0.1:0").expect("reserve local port");
    let port = reservation.local_addr().expect("reserved address").port();
    drop(reservation);
    let source = format!(
        "program CancellableAccept;\n\nuses Std.Net as Net; uses Std.Tasks as Tasks; uses Std.Time as Time;\n\nfunction WaitForCancellation(\n  ListenerValue: Net.Listener;\n  Token: Tasks.CancellationToken\n): string;\nbegin\n  case Net.AcceptWithCancellation(ListenerValue, Token) of\n    when Result.Ok(const Connection):\n    begin\n      discard Net.Close(Connection);\n      return 'accepted';\n    end;\n    when Result.Error(const Message): return Message;\n  end case;\nend function;\n\nbegin\n  case Net.Listen('127.0.0.1', {port}) of\n    when Result.Ok(const ListenerValue):\n    begin\n      var Source: Tasks.CancellationSource := Tasks.CreateCancellationSource();\n      var Token: Tasks.CancellationToken := Tasks.GetCancellationToken(Source);\n      var Waiting: task := go WaitForCancellation(ListenerValue, Token);\n      Time.Sleep(50);\n      if not Tasks.Cancel(Source) then panic('first cancellation did not change state'); end if;\n      if Tasks.Wait(Waiting) <> 'Network accept cancelled' then\n        panic('accept did not report cancellation'); end if;\n      discard Net.CloseListener(ListenerValue);\n    end;\n    when Result.Error(const Message): panic(Message);\n  end case;\nend program;"
    );

    assert_succeeds(&source);
}

#[test]
fn wait_all_keeps_register_task_results_available() {
    assert_succeeds(
        r#"program RegisterWaitAll;
uses Std.Tasks as Tasks;

function Work(Value: integer): integer;
begin
  return Value;
end function;

begin
  var A: task := go Work(20);
  var B: task := go Work(22);
  Tasks.WaitAll([A, B]);
  discard Tasks.Wait(A);
  discard Tasks.Wait(B);
end program;"#,
    );
}

#[test]
fn mutable_capture_cannot_cross_register_task_boundary() {
    let source = r#"program RegisterTaskBound;
uses Std.Tasks as Tasks;

function Make(): function(): integer;
begin
  mutable var Value: integer := 41;
  return function(): integer
  begin
    Value := Value + 1;
    return Value;
  end function;
end function;

begin
  var Work: function(): integer := Make();
  var T: task := go Work();
  discard Tasks.Wait(T);
end program;"#;
    let error = run_program(source).expect_err("runtime must reject task-bound closure");
    assert!(error.message.contains("task-bound"));
}

#[test]
fn bounded_channels_send_receive_close_and_drain_fifo() {
    assert_succeeds(
        r#"program BoundedChannels;

uses Std.Tasks as Tasks;

function Produce(Messages: channel of (integer)): boolean;
begin
  case Tasks.Send(Messages, 20) of
    when Result.Ok(_):
      begin
        null;
      end;
    when Result.Error(const Message):
      panic(Message);
  end case;

  case Tasks.Send(Messages, 22) of
    when Result.Ok(_):
      begin
        null;
      end;
    when Result.Error(const Message):
      panic(Message);
  end case;

  return Tasks.CloseChannel(Messages);
end function;

function Take(Messages: channel of (integer)): integer;
begin
  case Tasks.Receive(Messages) of
    when Result.Ok(const Value):
      return Value;
    when Result.Error(const Message):
      panic(Message);
  end case;
end function;

begin
  var Messages: channel of (integer) := Tasks.CreateChannel(1);
  var Producer: task := go Produce(Messages);
  if Take(Messages) <> 20 then
    panic('first channel value was not FIFO');
  end if;

  if Take(Messages) <> 22 then
    panic('second channel value was not FIFO');
  end if;

  if not Tasks.Wait(Producer) then
    panic('channel close was not first');
  end if;

  case Tasks.Receive(Messages) of
    when Result.Ok(_):
      panic('closed channel produced an extra value');
    when Result.Error(const Message):
      if Message <> 'Channel is closed' then
        panic(Message);
      end if;
  end case;

  if Tasks.CloseChannel(Messages) then
    panic('channel close was not idempotent');
  end if;
end program;
"#,
    );
}

#[test]
fn channel_creation_uses_argument_and_return_type_contexts() {
    assert_succeeds(
        r#"program ContextualChannels;
uses Std.Tasks as Tasks;

function MakeChannel(): channel of (integer);
begin
  return Tasks.CreateChannel(1);
end function;

function CloseChannelArgument(Messages: channel of (integer)): boolean;
begin
  return Tasks.CloseChannel(Messages);
end function;

begin
  if not CloseChannelArgument(Tasks.CreateChannel(1)) then
    panic('direct channel argument was not typed'); end if;
  var Messages: channel of (integer) := MakeChannel();
  if not Tasks.CloseChannel(Messages) then panic('returned channel was not typed'); end if;
end program;"#,
    );
}

#[test]
fn channel_non_blocking_and_timeout_operations_are_distinct() {
    assert_succeeds(
        r#"program ChannelWaitModes;

uses Std.Tasks as Tasks;

begin
  var Messages: channel of (integer) := Tasks.CreateChannel(1);
  case Tasks.TryReceive(Messages) of
    when Result.Ok(const MaybeValue):
      case MaybeValue of
        when Option.Some(_):
          panic('empty channel produced a value');
        when Option.None:
          begin
            null;
          end;
      end case;
    when Result.Error(const Message):
      panic(Message);
  end case;

  case Tasks.TrySend(Messages, 1) of
    when Result.Ok(const Sent):
      if not Sent then
        panic('first try-send did not send');
      end if;
    when Result.Error(const Message):
      panic(Message);
  end case;

  case Tasks.TrySend(Messages, 2) of
    when Result.Ok(const Sent):
      if Sent then
        panic('full channel accepted a value');
      end if;
    when Result.Error(const Message):
      panic(Message);
  end case;

  case Tasks.ReceiveWithTimeout(Messages, 0) of
    when Result.Ok(const Value):
      if Value <> 1 then
        panic('timeout receive changed FIFO order');
      end if;
    when Result.Error(const Message):
      panic(Message);
  end case;

  case Tasks.ReceiveWithTimeout(Messages, 1) of
    when Result.Ok(_):
      panic('empty channel did not time out');
    when Result.Error(const Message):
      if Message <> 'Channel receive timed out' then
        panic(Message);
      end if;
  end case;

  case Tasks.Send(Messages, 3) of
    when Result.Ok(_):
      begin
        null;
      end;
    when Result.Error(const Message):
      panic(Message);
  end case;

  case Tasks.SendWithTimeout(Messages, 4, 1) of
    when Result.Ok(_):
      panic('full channel did not time out');
    when Result.Error(const Message):
      if Message <> 'Channel send timed out' then
        panic(Message);
      end if;
  end case;
end program;
"#,
    );
}

#[test]
fn channel_timeout_rejects_negative_milliseconds() {
    let error = run_program(
        r#"program InvalidChannelTimeout;
uses Std.Tasks as Tasks;
begin
  var Messages: channel of (integer) := Tasks.CreateChannel(1);
  discard Tasks.ReceiveWithTimeout(Messages, -1);
end program;"#,
    )
    .expect_err("negative channel timeout must fail");
    assert!(error.message.contains("non-negative timeout"));
}

#[test]
fn cancellable_channel_send_and_receive_report_distinct_errors() {
    assert_succeeds(
        r#"program CancellableChannels;

uses Std.Tasks as Tasks;
uses Std.Time as Time;

function BlockedSend(Messages: channel of (integer); Token: Tasks.CancellationToken): string;
begin
  case Tasks.SendWithCancellation(Messages, 2, Token) of
    when Result.Ok(_):
      return 'sent';
    when Result.Error(const Message):
      return Message;
  end case;
end function;

function BlockedReceive(Messages: channel of (integer); Token: Tasks.CancellationToken): string;
begin
  case Tasks.ReceiveWithCancellation(Messages, Token) of
    when Result.Ok(_):
      return 'received';
    when Result.Error(const Message):
      return Message;
  end case;
end function;

begin
  var Full: channel of (integer) := Tasks.CreateChannel(1);
  case Tasks.Send(Full, 1) of
    when Result.Ok(_):
      begin
        null;
      end;
    when Result.Error(const Message):
      panic(Message);
  end case;
  var SendSource: Tasks.CancellationSource := Tasks.CreateCancellationSource();
  var Sending: task := go BlockedSend(Full, Tasks.GetCancellationToken(SendSource));
  Time.Sleep(20);
  discard Tasks.Cancel(SendSource);
  if Tasks.Wait(Sending) <> 'Channel send was cancelled' then
    panic('send cancellation mismatch');
  end if;
  var Empty: channel of (integer) := Tasks.CreateChannel(1);
  var ReceiveSource: Tasks.CancellationSource := Tasks.CreateCancellationSource();
  var Receiving: task := go BlockedReceive(Empty, Tasks.GetCancellationToken(ReceiveSource));
  Time.Sleep(20);
  discard Tasks.Cancel(ReceiveSource);
  if Tasks.Wait(Receiving) <> 'Channel receive was cancelled' then
    panic('receive cancellation mismatch');
  end if;
end program;
"#,
    );
}
