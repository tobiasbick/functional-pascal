use super::*;

mod network_connect;
mod network_read;
mod network_write;
mod task_results;
mod wait_any;

#[test]
fn task_spawn_with_arguments_keeps_loop_branch_addresses_aligned() {
    assert_succeeds(
        "\
program RegisterTaskArgumentLoop;
uses Std.Arrays, Std.Tasks;
function Worker(Value: integer): integer;
begin
  return Value + 1;
end function;
begin
  var Tasks: array of task := [];
  for Index: integer := 1 to 8 do
  begin
    Push(Tasks, go Worker(Index));
  end; end for;
  WaitAll(Tasks);
  if Length(Tasks) <> 8 then panic('task loop count mismatch'); end if;
end.",
    );
}

#[test]
fn retained_task_spawn_and_wait_execute() {
    let execution = assert_succeeds(
        "\
program RegisterTasks;
uses Std.Console, Std.Tasks;

function Add(A: integer; B: integer): integer;
begin
  return A + B;
end function;

begin
  const T: task := go Add(20, 22);
  Std.Console.WriteLn(Std.Tasks.Wait(T));
end.",
    );
    assert_eq!(execution.value, fpas_bytecode::Value::Unit);
}

#[test]
fn detached_task_executes_on_register_pool() {
    assert_succeeds(
        "\
program RegisterDetached;
uses Std.Console;

procedure Work();
begin
  Std.Console.WriteLn('worker');
end procedure;

begin
  go Work();
end.",
    );
}

#[test]
fn timeslice_preserves_nested_frames_and_live_aggregate_registers() {
    assert_succeeds(
        "\
program RegisterTaskFrames;
uses Std.Tasks;

function Burn(Count: integer): integer;
begin
  var I: integer := 0;
  while I < Count do
    I := I + 1; end while;
  return I;
end function;

function Work(): integer;
begin
  const Values: array of integer := [40, 2];
  return Burn(700) - 700 + Values[0] + Values[1];
end function;

begin
  const T: task := go Work();
  if Std.Tasks.Wait(T) <> 42 then panic('task state was not restored'); end if;
end.",
    );
}

#[test]
fn cooperative_sleep_releases_register_pool_worker() {
    assert_succeeds(
        "\
program RegisterTaskSleep;
uses Std.Tasks, Std.Time;

function Work(Value: integer): integer;
begin
  Std.Time.Sleep(1);
  return Value;
end function;

begin
  const A: task := go Work(42);
  discard Std.Tasks.Wait(A);
end.",
    );
}

#[test]
fn cancellation_token_interrupts_network_accept_end_to_end() {
    let reservation = std::net::TcpListener::bind("127.0.0.1:0").expect("reserve local port");
    let port = reservation.local_addr().expect("reserved address").port();
    drop(reservation);
    let source = format!(
        "\
program CancellableAccept;

uses Std.Net, Std.Tasks, Std.Time;

function WaitForCancellation(
  ListenerValue: Std.Net.Listener;
  Token: Std.Tasks.CancellationToken
): string;
begin
  case Std.Net.AcceptWithCancellation(ListenerValue, Token) of
    when Ok(Connection):
    begin
      discard Std.Net.Close(Connection);
      return 'accepted';
    end;
    when Error(Message): return Message;
  end case;
end function;

begin
  case Std.Net.Listen('127.0.0.1', {port}) of
    when Ok(ListenerValue):
    begin
      const Source: Std.Tasks.CancellationSource := Std.Tasks.CreateCancellationSource();
      const Token: Std.Tasks.CancellationToken := Std.Tasks.GetCancellationToken(Source);
      const Waiting: task := go WaitForCancellation(ListenerValue, Token);
      Std.Time.Sleep(50);
      if not Std.Tasks.Cancel(Source) then panic('first cancellation did not change state'); end if;
      if Std.Tasks.Wait(Waiting) <> 'Network accept cancelled' then
        panic('accept did not report cancellation'); end if;
      discard Std.Net.CloseListener(ListenerValue);
    end;
    when Error(Message): panic(Message);
  end case;
end."
    );

    assert_succeeds(&source);
}

#[test]
fn wait_all_keeps_register_task_results_available() {
    assert_succeeds(
        "\
program RegisterWaitAll;
uses Std.Tasks;

function Work(Value: integer): integer;
begin
  return Value;
end function;

begin
  const A: task := go Work(20);
  const B: task := go Work(22);
  Std.Tasks.WaitAll([A, B]);
  discard Std.Tasks.Wait(A);
  discard Std.Tasks.Wait(B);
end.",
    );
}

#[test]
fn mutable_capture_cannot_cross_register_task_boundary() {
    let source = "\
program RegisterTaskBound;
uses Std.Tasks;

function Make(): function(): integer;
begin
  var Value: integer := 41;
  return function(): integer
  begin
    Value := Value + 1;
    return Value;
  end function;
end function;

begin
  const Work: function(): integer := Make();
  const T: task := go Work();
  discard Std.Tasks.Wait(T);
end.";
    let error = run_program(source).expect_err("runtime must reject task-bound closure");
    assert!(error.message.contains("task-bound"));
}

#[test]
fn bounded_channels_send_receive_close_and_drain_fifo() {
    assert_succeeds(
        "\
program BoundedChannels;
uses Std.Tasks;

function Produce(Messages: channel of integer): boolean;
begin
  case Send(Messages, 20) of
    when Ok(_): begin end;
    when Error(Message): panic(Message);
  end case;
  case Send(Messages, 22) of
    when Ok(_): begin end;
    when Error(Message): panic(Message);
  end case;
  return CloseChannel(Messages);
end function;

function Take(Messages: channel of integer): integer;
begin
  case Receive(Messages) of
    when Ok(Value): return Value;
    when Error(Message): panic(Message);
  end case;
end function;

begin
  const Messages: channel of integer := CreateChannel(1);
  const Producer: task := go Produce(Messages);
  if Take(Messages) <> 20 then panic('first channel value was not FIFO'); end if;
  if Take(Messages) <> 22 then panic('second channel value was not FIFO'); end if;
  if not Wait(Producer) then panic('channel close was not first'); end if;
  case Receive(Messages) of
    when Ok(_): panic('closed channel produced an extra value');
    when Error(Message):
      if Message <> 'Channel is closed' then panic(Message); end if;
  end case;
  if CloseChannel(Messages) then panic('channel close was not idempotent'); end if;
end.",
    );
}

#[test]
fn channel_creation_uses_argument_and_return_type_contexts() {
    assert_succeeds(
        "\
program ContextualChannels;
uses Std.Tasks;

function MakeChannel(): channel of integer;
begin
  return CreateChannel(1);
end function;

function CloseChannelArgument(Messages: channel of integer): boolean;
begin
  return CloseChannel(Messages);
end function;

begin
  if not CloseChannelArgument(CreateChannel(1)) then
    panic('direct channel argument was not typed'); end if;
  const Messages: channel of integer := MakeChannel();
  if not CloseChannel(Messages) then panic('returned channel was not typed'); end if;
end.",
    );
}

#[test]
fn channel_non_blocking_and_timeout_operations_are_distinct() {
    assert_succeeds(
        "\
program ChannelWaitModes;
uses Std.Tasks;

begin
  const Messages: channel of integer := CreateChannel(1);
  case TryReceive(Messages) of
    when Ok(MaybeValue):
      case MaybeValue of
        when Some(_): panic('empty channel produced a value');
        when None: begin end;
      end case;
    when Error(Message): panic(Message);
  end case;
  case TrySend(Messages, 1) of
    when Ok(Sent): if not Sent then panic('first try-send did not send'); end if;
    when Error(Message): panic(Message);
  end case;
  case TrySend(Messages, 2) of
    when Ok(Sent): if Sent then panic('full channel accepted a value'); end if;
    when Error(Message): panic(Message);
  end case;
  case ReceiveWithTimeout(Messages, 0) of
    when Ok(Value): if Value <> 1 then panic('timeout receive changed FIFO order'); end if;
    when Error(Message): panic(Message);
  end case;
  case ReceiveWithTimeout(Messages, 1) of
    when Ok(_): panic('empty channel did not time out');
    when Error(Message):
      if Message <> 'Channel receive timed out' then panic(Message); end if;
  end case;
  case Send(Messages, 3) of
    when Ok(_): begin end;
    when Error(Message): panic(Message);
  end case;
  case SendWithTimeout(Messages, 4, 1) of
    when Ok(_): panic('full channel did not time out');
    when Error(Message):
      if Message <> 'Channel send timed out' then panic(Message); end if;
  end case;
end.",
    );
}

#[test]
fn channel_timeout_rejects_negative_milliseconds() {
    let error = run_program(
        "\
program InvalidChannelTimeout;
uses Std.Tasks;
begin
  const Messages: channel of integer := CreateChannel(1);
  discard ReceiveWithTimeout(Messages, -1);
end.",
    )
    .expect_err("negative channel timeout must fail");
    assert!(error.message.contains("non-negative timeout"));
}

#[test]
fn cancellable_channel_send_and_receive_report_distinct_errors() {
    assert_succeeds(
        "\
program CancellableChannels;
uses Std.Tasks, Std.Time;

function BlockedSend(
  Messages: channel of integer;
  Token: CancellationToken
): string;
begin
  case SendWithCancellation(Messages, 2, Token) of
    when Ok(_): return 'sent';
    when Error(Message): return Message;
  end case;
end function;

function BlockedReceive(
  Messages: channel of integer;
  Token: CancellationToken
): string;
begin
  case ReceiveWithCancellation(Messages, Token) of
    when Ok(_): return 'received';
    when Error(Message): return Message;
  end case;
end function;

begin
  const Full: channel of integer := CreateChannel(1);
  case Send(Full, 1) of
    when Ok(_): begin end;
    when Error(Message): panic(Message);
  end case;
  const SendSource: CancellationSource := CreateCancellationSource();
  const Sending: task := go BlockedSend(Full, GetCancellationToken(SendSource));
  Sleep(20);
  discard Cancel(SendSource);
  if Wait(Sending) <> 'Channel send was cancelled' then panic('send cancellation mismatch'); end if;

  const Empty: channel of integer := CreateChannel(1);
  const ReceiveSource: CancellationSource := CreateCancellationSource();
  const Receiving: task := go BlockedReceive(Empty, GetCancellationToken(ReceiveSource));
  Sleep(20);
  discard Cancel(ReceiveSource);
  if Wait(Receiving) <> 'Channel receive was cancelled' then
    panic('receive cancellation mismatch'); end if;
end.",
    );
}
