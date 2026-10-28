use super::*;

mod network_connect;
mod network_read;
mod network_write;
mod static_capabilities;
mod task_results;
mod task_scopes;
mod wait_any;

#[test]
fn task_spawn_with_arguments_keeps_loop_branch_addresses_aligned() {
    assert_succeeds(
        "program RegisterTaskArgumentLoop;\nuses Std.Tasks as StdTasks;\nfunction Worker(Value: integer): integer;\nbegin\n  return Value + 1;\nend function;\nbegin scope \n  var Tasks: array of task of integer := [];\n  for Index: integer := 1 to 8 do\n  begin\n    Tasks.PushInPlace(go Worker(Index));\n  end; end for;\n  StdTasks.WaitAll(Tasks);\n  if Tasks.Length() <> 8 then panic('task loop count mismatch'); end if;\n end scope; end.",
    );
}

#[test]
fn retained_task_spawn_and_wait_execute() {
    let execution = assert_succeeds(
        "program RegisterTasks;\nuses Std.Console as Console, Std.Tasks as Tasks;\n\nfunction Add(A: integer; B: integer): integer;\nbegin\n  return A + B;\nend function;\n\nbegin scope\n  const T: task of integer := go Add(20, 22);\n  Console.WriteLn(Tasks.Wait(T));\n end scope; end.",
    );
    assert_eq!(
        execution.value.try_to_data().expect("pure return value"),
        fpas_runtime::Value::Unit
    );
}

#[test]
fn detached_task_executes_on_register_pool() {
    assert_succeeds(
        "program RegisterDetached;\nuses Std.Console as Console;\n\nprocedure Work();\nbegin\n  Console.WriteLn('worker');\nend procedure;\n\nbegin scope\n  discard go Work();\n end scope; end.",
    );
}

#[test]
fn timeslice_preserves_nested_frames_and_live_aggregate_registers() {
    assert_succeeds(
        "program RegisterTaskFrames;\nuses Std.Tasks as Tasks;\n\nfunction Burn(Count: integer): integer;\nbegin\n  var I: integer := 0;\n  while I < Count do\n    I := I + 1; end while;\n  return I;\nend function;\n\nfunction Work(): integer;\nbegin\n  const Values: array of integer := [40, 2];\n  return Burn(700) - 700 + Values[0] + Values[1];\nend function;\n\nbegin scope\n  const T: task of integer := go Work();\n  if Tasks.Wait(T) <> 42 then panic('task state was not restored'); end if;\n end scope; end.",
    );
}

#[test]
fn cooperative_sleep_releases_register_pool_worker() {
    assert_succeeds(
        "program RegisterTaskSleep;\nuses Std.Tasks as Tasks, Std.Time as Time;\n\nfunction Work(Value: integer): integer;\nbegin\n  Time.Sleep(1);\n  return Value;\nend function;\n\nbegin scope\n  const A: task of integer := go Work(42);\n  discard Tasks.Wait(A);\n end scope; end.",
    );
}

#[test]
fn cancellation_token_interrupts_network_accept_end_to_end() {
    let reservation = std::net::TcpListener::bind("127.0.0.1:0").expect("reserve local port");
    let port = reservation.local_addr().expect("reserved address").port();
    drop(reservation);
    let source = format!(
        "program CancellableAccept;\n\nuses Std.Net as Net, Std.Tasks as Tasks, Std.Time as Time;\n\nfunction WaitForCancellation(\n  ListenerValue: Net.Listener;\n  Token: Tasks.CancellationToken\n): option of Net.NetError;\nbegin\n  case Net.AcceptWithCancellation(ListenerValue, Token) of\n    when Ok(const Connection):\n    begin\n      discard Net.Close(Connection);\n      return None;\n    end;\n    when Error(const Message): return Some(Message);\n  end case;\nend function;\n\nbegin scope\n  case Net.Listen('127.0.0.1', {port}) of\n    when Ok(const ListenerValue):\n    begin\n      const Source: Tasks.CancellationSource := Tasks.CreateCancellationSource();\n      const Token: Tasks.CancellationToken := Tasks.GetCancellationToken(Source);\n      const Waiting: task of option of Net.NetError := go WaitForCancellation(ListenerValue, Token);\n      Time.Sleep(50);\n      if not Tasks.Cancel(Source) then panic('first cancellation did not change state'); end if;\n      if not (Tasks.Wait(Waiting) is Some(Net.NetError.Cancelled('Std.Net.AcceptWithCancellation'))) then\n        panic('accept did not report cancellation'); end if;\n      discard Net.CloseListener(ListenerValue);\n    end;\n    when Error(const Message): panic(Message.Message());\n  end case;\n end scope; end."
    );

    assert_succeeds(&source);
}

#[test]
fn wait_all_keeps_register_task_results_available() {
    assert_succeeds(
        "program RegisterWaitAll;\nuses Std.Tasks as Tasks;\n\nfunction Work(Value: integer): integer;\nbegin\n  return Value;\nend function;\n\nbegin scope\n  const A: task of integer := go Work(20);\n  const B: task of integer := go Work(22);\n  Tasks.WaitAll([A, B]);\n  discard Tasks.Wait(A);\n  discard Tasks.Wait(B);\n end scope; end.",
    );
}

#[test]
fn mutable_capture_cannot_cross_register_task_boundary() {
    let source = "program RegisterTaskBound;\nuses Std.Tasks as Tasks;\n\nfunction Make(): function(): integer;\nbegin\n  var Value: integer := 41;\n  return function(): integer\n  begin\n    Value := Value + 1;\n    return Value;\n  end function;\nend function;\n\nprocedure Start(Work: function(): integer);\nbegin scope\n  const T: task of integer := go Work();\n  discard Tasks.Wait(T);\nend scope; end procedure;\n\nbegin Start(Make()); end.";
    let error = run_program(source).expect_err("runtime must reject task-bound closure");
    assert!(error.message.contains("task-bound"));
}

#[test]
fn bounded_channels_send_receive_close_and_drain_fifo() {
    assert_succeeds(
        "program BoundedChannels;\nuses Std.Tasks as Tasks;\n\nfunction Produce(Messages: channel of integer): boolean;\nbegin\n  case Tasks.Send(Messages, 20) of\n    when Ok(_): begin end;\n    when Error(const Message): panic(Message.Message());\n  end case;\n  case Tasks.Send(Messages, 22) of\n    when Ok(_): begin end;\n    when Error(const Message): panic(Message.Message());\n  end case;\n  return Tasks.CloseChannel(Messages);\nend function;\n\nfunction Take(Messages: channel of integer): integer;\nbegin\n  case Tasks.Receive(Messages) of\n    when Ok(const Value): return Value;\n    when Error(const Message): panic(Message.Message());\n  end case;\nend function;\n\nbegin scope\n  const Messages: channel of integer := Tasks.CreateChannel(1);\n  const Producer: task of boolean := go Produce(Messages);\n  if Take(Messages) <> 20 then panic('first channel value was not FIFO'); end if;\n  if Take(Messages) <> 22 then panic('second channel value was not FIFO'); end if;\n  if not Tasks.Wait(Producer) then panic('channel close was not first'); end if;\n  case Tasks.Receive(Messages) of\n    when Ok(_): panic('closed channel produced an extra value');\n    when Error(const Message):\n      if not (Message is Tasks.TaskError.Closed(_)) then panic(Message.Message()); end if;\n  end case;\n  if Tasks.CloseChannel(Messages) then panic('channel close was not idempotent'); end if;\n end scope; end.",
    );
}

#[test]
fn channel_creation_uses_argument_and_return_type_contexts() {
    assert_succeeds(
        "program ContextualChannels;\nuses Std.Tasks as Tasks;\n\nfunction MakeChannel(): channel of integer;\nbegin\n  return Tasks.CreateChannel(1);\nend function;\n\nfunction CloseChannelArgument(Messages: channel of integer): boolean;\nbegin\n  return Tasks.CloseChannel(Messages);\nend function;\n\nbegin\n  if not CloseChannelArgument(Tasks.CreateChannel(1)) then\n    panic('direct channel argument was not typed'); end if;\n  const Messages: channel of integer := MakeChannel();\n  if not Tasks.CloseChannel(Messages) then panic('returned channel was not typed'); end if;\nend.",
    );
}

#[test]
fn channel_non_blocking_and_timeout_operations_are_distinct() {
    assert_succeeds(
        "program ChannelWaitModes;\nuses Std.Tasks as Tasks;\n\nbegin\n  const Messages: channel of integer := Tasks.CreateChannel(1);\n  case Tasks.TryReceive(Messages) of\n    when Ok(const MaybeValue):\n      case MaybeValue of\n        when Some(_): panic('empty channel produced a value');\n        when None: begin end;\n      end case;\n    when Error(const Message): panic(Message.Message());\n  end case;\n  case Tasks.TrySend(Messages, 1) of\n    when Ok(const Sent): if not Sent then panic('first try-send did not send'); end if;\n    when Error(const Message): panic(Message.Message());\n  end case;\n  case Tasks.TrySend(Messages, 2) of\n    when Ok(const Sent): if Sent then panic('full channel accepted a value'); end if;\n    when Error(const Message): panic(Message.Message());\n  end case;\n  case Tasks.ReceiveWithTimeout(Messages, 0) of\n    when Ok(const Value): if Value <> 1 then panic('timeout receive changed FIFO order'); end if;\n    when Error(const Message): panic(Message.Message());\n  end case;\n  case Tasks.ReceiveWithTimeout(Messages, 1) of\n    when Ok(_): panic('empty channel did not time out');\n    when Error(const Message):\n      if not (Message is Tasks.TaskError.Timeout('Std.Tasks.ReceiveWithTimeout')) then panic(Message.Message()); end if;\n  end case;\n  case Tasks.Send(Messages, 3) of\n    when Ok(_): begin end;\n    when Error(const Message): panic(Message.Message());\n  end case;\n  case Tasks.SendWithTimeout(Messages, 4, 1) of\n    when Ok(_): panic('full channel did not time out');\n    when Error(const Message):\n      if not (Message is Tasks.TaskError.Timeout('Std.Tasks.SendWithTimeout')) then panic(Message.Message()); end if;\n  end case;\nend.",
    );
}

#[test]
fn channel_timeout_rejects_negative_milliseconds() {
    assert_succeeds(
        "program InvalidChannelTimeout; uses Std.Tasks as Tasks, Std.Test as Test; begin const Messages: channel of integer := Tasks.CreateChannel(1); Test.AssertTrue(Tasks.ReceiveWithTimeout(Messages, -1) is Error(Tasks.TaskError.InvalidArgument('TimeoutMillis', _))); end.",
    );
}

#[test]
fn cancellable_channel_send_and_receive_report_distinct_errors() {
    assert_succeeds(
        "program CancellableChannels;\nuses Std.Tasks as Tasks, Std.Time as Time;\n\nfunction BlockedSend(\n  Messages: channel of integer;\n  Token: Tasks.CancellationToken\n): option of Tasks.TaskError;\nbegin\n  case Tasks.SendWithCancellation(Messages, 2, Token) of\n    when Ok(_): return None;\n    when Error(const Message): return Some(Message);\n  end case;\nend function;\n\nfunction BlockedReceive(\n  Messages: channel of integer;\n  Token: Tasks.CancellationToken\n): option of Tasks.TaskError;\nbegin\n  case Tasks.ReceiveWithCancellation(Messages, Token) of\n    when Ok(_): return None;\n    when Error(const Message): return Some(Message);\n  end case;\nend function;\n\nbegin scope\n  const Full: channel of integer := Tasks.CreateChannel(1);\n  case Tasks.Send(Full, 1) of\n    when Ok(_): begin end;\n    when Error(const Message): panic(Message.Message());\n  end case;\n  const SendSource: Tasks.CancellationSource := Tasks.CreateCancellationSource();\n  const Sending: task of option of Tasks.TaskError := go BlockedSend(Full, Tasks.GetCancellationToken(SendSource));\n  Time.Sleep(20);\n  discard Tasks.Cancel(SendSource);\n  if not (Tasks.Wait(Sending) is Some(Tasks.TaskError.Cancelled('Std.Tasks.SendWithCancellation'))) then panic('send cancellation mismatch'); end if;\n\n  const Empty: channel of integer := Tasks.CreateChannel(1);\n  const ReceiveSource: Tasks.CancellationSource := Tasks.CreateCancellationSource();\n  const Receiving: task of option of Tasks.TaskError := go BlockedReceive(Empty, Tasks.GetCancellationToken(ReceiveSource));\n  Time.Sleep(20);\n  discard Tasks.Cancel(ReceiveSource);\n  if not (Tasks.Wait(Receiving) is Some(Tasks.TaskError.Cancelled('Std.Tasks.ReceiveWithCancellation'))) then\n    panic('receive cancellation mismatch'); end if;\n end scope; end.",
    );
}
