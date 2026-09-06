use super::*;

#[test]
fn task_spawn_with_arguments_keeps_loop_branch_addresses_aligned() {
    assert_succeeds(
        "\
program RegisterTaskArgumentLoop;
uses Std.Array, Std.Task;
function Worker(Value: integer): integer;
begin
  return Value + 1
end;
begin
  mutable var Tasks: array of task := [];
  for Index: integer := 1 to 8 do
  begin
    Push(Tasks, go Worker(Index))
  end;
  WaitAll(Tasks);
  if Length(Tasks) <> 8 then panic('task loop count mismatch')
end.",
    );
}

#[test]
fn retained_task_spawn_and_wait_execute() {
    let execution = assert_succeeds(
        "\
program RegisterTasks;

function Add(A: integer; B: integer): integer;
begin
  return A + B
end;

begin
  var T: task := go Add(20, 22);
  Std.Console.WriteLn(Std.Task.Wait(T))
end.",
    );
    assert_eq!(execution.value, fpas_bytecode::Value::Unit);
}

#[test]
fn detached_task_executes_on_register_pool() {
    assert_succeeds(
        "\
program RegisterDetached;

procedure Work();
begin
  Std.Console.WriteLn('worker')
end;

begin
  go Work()
end.",
    );
}

#[test]
fn timeslice_preserves_nested_frames_and_live_aggregate_registers() {
    assert_succeeds(
        "\
program RegisterTaskFrames;

function Burn(Count: integer): integer;
begin
  mutable var I: integer := 0;
  while I < Count do
    I := I + 1;
  return I
end;

function Work(): integer;
begin
  var Values: array of integer := [40, 2];
  return Burn(700) - 700 + Values[0] + Values[1]
end;

begin
  var T: task := go Work();
  if Std.Task.Wait(T) <> 42 then panic('task state was not restored')
end.",
    );
}

#[test]
fn cooperative_sleep_releases_register_pool_worker() {
    assert_succeeds(
        "\
program RegisterTaskSleep;

function Work(Value: integer): integer;
begin
  Std.Time.Sleep(1);
  return Value
end;

begin
  var A: task := go Work(42);
  Std.Task.Wait(A)
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

uses Std.Net, Std.Task, Std.Time;

function WaitForCancellation(
  ListenerValue: Std.Net.Listener;
  Token: Std.Task.CancellationToken
): string;
begin
  case Std.Net.AcceptWithCancellation(ListenerValue, Token) of
    Ok(Connection):
    begin
      Std.Net.Close(Connection);
      return 'accepted'
    end;
    Error(Message): return Message
  end
end;

begin
  case Std.Net.Listen('127.0.0.1', {port}) of
    Ok(ListenerValue):
    begin
      var Source: Std.Task.CancellationSource := Std.Task.CreateCancellationSource();
      var Token: Std.Task.CancellationToken := Std.Task.GetCancellationToken(Source);
      var Waiting: task := go WaitForCancellation(ListenerValue, Token);
      Std.Time.Sleep(50);
      if not Std.Task.Cancel(Source) then panic('first cancellation did not change state');
      if Std.Task.Wait(Waiting) <> 'Network accept cancelled' then
        panic('accept did not report cancellation');
      Std.Net.CloseListener(ListenerValue)
    end;
    Error(Message): panic(Message)
  end
end."
    );

    assert_succeeds(&source);
}

#[test]
fn wait_all_keeps_register_task_results_available() {
    assert_succeeds(
        "\
program RegisterWaitAll;

function Work(Value: integer): integer;
begin
  return Value
end;

begin
  var A: task := go Work(20);
  var B: task := go Work(22);
  Std.Task.WaitAll([A, B]);
  Std.Task.Wait(A);
  Std.Task.Wait(B)
end.",
    );
}

#[test]
fn mutable_capture_cannot_cross_register_task_boundary() {
    let source = "\
program RegisterTaskBound;

function Make(): function(): integer;
begin
  mutable var Value: integer := 41;
  return function(): integer
  begin
    Value := Value + 1;
    return Value
  end
end;

begin
  var Work: function(): integer := Make();
  var T: task := go Work();
  Std.Task.Wait(T)
end.";
    let error = run_program(source).expect_err("runtime must reject task-bound closure");
    assert!(error.message.contains("task-bound"));
}

#[test]
fn bounded_channels_send_receive_close_and_drain_fifo() {
    assert_succeeds(
        "\
program BoundedChannels;
uses Std.Task;

function Produce(Messages: channel of integer): boolean;
begin
  case Send(Messages, 20) of
    Ok(_): begin end;
    Error(Message): panic(Message)
  end;
  case Send(Messages, 22) of
    Ok(_): begin end;
    Error(Message): panic(Message)
  end;
  return CloseChannel(Messages)
end;

function Take(Messages: channel of integer): integer;
begin
  case Receive(Messages) of
    Ok(Value): return Value;
    Error(Message): panic(Message)
  end
end;

begin
  var Messages: channel of integer := CreateChannel(1);
  var Producer: task := go Produce(Messages);
  if Take(Messages) <> 20 then panic('first channel value was not FIFO');
  if Take(Messages) <> 22 then panic('second channel value was not FIFO');
  if not Wait(Producer) then panic('channel close was not first');
  case Receive(Messages) of
    Ok(_): panic('closed channel produced an extra value');
    Error(Message):
      if Message <> 'Channel is closed' then panic(Message)
  end;
  if CloseChannel(Messages) then panic('channel close was not idempotent')
end.",
    );
}

#[test]
fn channel_creation_uses_argument_and_return_type_contexts() {
    assert_succeeds(
        "\
program ContextualChannels;
uses Std.Task;

function MakeChannel(): channel of integer;
begin
  return CreateChannel(1)
end;

function CloseChannelArgument(Messages: channel of integer): boolean;
begin
  return CloseChannel(Messages)
end;

begin
  if not CloseChannelArgument(CreateChannel(1)) then
    panic('direct channel argument was not typed');
  var Messages: channel of integer := MakeChannel();
  if not CloseChannel(Messages) then panic('returned channel was not typed')
end.",
    );
}

#[test]
fn cancellable_channel_send_and_receive_report_distinct_errors() {
    assert_succeeds(
        "\
program CancellableChannels;
uses Std.Task, Std.Time;

function BlockedSend(
  Messages: channel of integer;
  Token: CancellationToken
): string;
begin
  case SendWithCancellation(Messages, 2, Token) of
    Ok(_): return 'sent';
    Error(Message): return Message
  end
end;

function BlockedReceive(
  Messages: channel of integer;
  Token: CancellationToken
): string;
begin
  case ReceiveWithCancellation(Messages, Token) of
    Ok(_): return 'received';
    Error(Message): return Message
  end
end;

begin
  var Full: channel of integer := CreateChannel(1);
  case Send(Full, 1) of
    Ok(_): begin end;
    Error(Message): panic(Message)
  end;
  var SendSource: CancellationSource := CreateCancellationSource();
  var Sending: task := go BlockedSend(Full, GetCancellationToken(SendSource));
  Sleep(20);
  Cancel(SendSource);
  if Wait(Sending) <> 'Channel send was cancelled' then panic('send cancellation mismatch');

  var Empty: channel of integer := CreateChannel(1);
  var ReceiveSource: CancellationSource := CreateCancellationSource();
  var Receiving: task := go BlockedReceive(Empty, GetCancellationToken(ReceiveSource));
  Sleep(20);
  Cancel(ReceiveSource);
  if Wait(Receiving) <> 'Channel receive was cancelled' then
    panic('receive cancellation mismatch')
end.",
    );
}
