//! Main-task channel and task waits: progress through the pool without running queued tasks inline.

fn run_with_workers(source: &str, workers: usize) {
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    let mut vm = crate::execution::vm::Vm::new(fpas_compiler::compile(&program).expect("compile"));
    vm.pool_size = workers;
    vm.run().expect("program completes");
}

fn run_with_one_worker(source: &str) {
    run_with_workers(source, 1);
}

/// A queued task that blocks on socket I/O must not run on the main task while it waits on a
/// channel: the main task may be the peer that task waits for, which would deadlock until the
/// task's own I/O times out. Two blocking tasks guarantee one stays queued behind the worker.
#[test]
fn timed_channel_wait_does_not_run_a_blocking_queued_task_inline() {
    run_with_one_worker(
        r#"program TimedWaitDoesNotHelp;
uses Std.Net as Net, Std.Tasks as Tasks, Std.Time as Time;
function BlockingRead(ListenerValue: Net.Listener; Token: Tasks.CancellationToken): boolean;
begin
  const Client: Net.Connection := Net.Accept(ListenerValue).Unwrap();
  const Configured: boolean := Net.SetTimeout(Client, 1500).Unwrap();
  const Ignored: result of (array of integer, Net.NetError) := Net.ReceiveBytesWithCancellation(Client, 1, Token);
  const Closed: boolean := Net.Close(Client).Unwrap();
  return true;
end function;
begin scope
  const Source: Tasks.CancellationSource := Tasks.CreateCancellationSource();
  const Token: Tasks.CancellationToken := Tasks.GetCancellationToken(Source);
  const FirstListener: Net.Listener := Net.Listen('127.0.0.1', 0).Unwrap();
  const SecondListener: Net.Listener := Net.Listen('127.0.0.1', 0).Unwrap();
  const FirstClient: Net.Connection := Net.Connect('127.0.0.1', Net.ListenerLocalAddress(FirstListener).Unwrap().Port, 1000).Unwrap();
  const SecondClient: Net.Connection := Net.Connect('127.0.0.1', Net.ListenerLocalAddress(SecondListener).Unwrap().Port, 1000).Unwrap();
  const First: task of boolean := go BlockingRead(FirstListener, Token);
  const Second: task of boolean := go BlockingRead(SecondListener, Token);
  const Events: channel of integer := Tasks.CreateChannel(1);
  var Started: integer := Time.TimestampMillis();
  const Outcome: result of (integer, Tasks.TaskError) := Tasks.ReceiveWithTimeout(Events, 100);
  if Time.TimestampMillis() - Started > 1000 then panic('timed receive ran a blocking task inline'); end if;
  if (Outcome is Ok(_)) then panic('nothing was sent'); end if;
  Started := Time.TimestampMillis();
  const Full: boolean := Tasks.SendWithTimeout(Events, 1, 100).Unwrap();
  const Blocked: result of (boolean, Tasks.TaskError) := Tasks.SendWithTimeout(Events, 2, 100);
  if Time.TimestampMillis() - Started > 1000 then panic('timed send ran a blocking task inline'); end if;
  if (Blocked is Ok(_)) then panic('the channel was full'); end if;
  if not Tasks.Wait(First) then panic('first'); end if;
  if not Tasks.Wait(Second) then panic('second'); end if;
 end scope; end."#,
    );
}

/// Without inline helping, the pool worker still serves main-task producer/consumer waits.
#[test]
fn untimed_channel_waits_progress_through_the_pool_worker() {
    run_with_one_worker(
        r#"program PoolServesMainChannelWaits;
uses Std.Tasks as Tasks;
function Doubler(Requests: channel of integer; Replies: channel of integer): integer;
begin
  var Count: integer := 0;
  for Index: integer := 1 to 50 do
  begin
    const Value: integer := Tasks.Receive(Requests).Unwrap();
    const Sent: boolean := Tasks.Send(Replies, Value * 2).Unwrap();
    Count := Count + 1;
  end; end for;
  return Count;
end function;
begin scope
  const Requests: channel of integer := Tasks.CreateChannel(1);
  const Replies: channel of integer := Tasks.CreateChannel(1);
  const Worker: task of integer := go Doubler(Requests, Replies);
  for Index: integer := 1 to 50 do
  begin
    const Sent: boolean := Tasks.Send(Requests, Index).Unwrap();
    if Tasks.Receive(Replies).Unwrap() <> Index * 2 then panic('reply'); end if;
  end; end for;
  if Tasks.Wait(Worker) <> 50 then panic('count'); end if;
 end scope; end."#,
    );
}

/// While the main task waits for `Quick`, it must not run the queued `Reader`, which waits for a
/// byte the main task sends only after that wait returns. Two busy tasks keep both workers
/// occupied so `Reader` and `Quick` are still queued when the main task starts waiting.
#[test]
fn task_wait_does_not_run_a_queued_task_that_waits_for_the_main_task() {
    run_with_workers(
        r#"program TaskWaitDoesNotHelp;
uses Std.Net as Net, Std.Tasks as Tasks, Std.Time as Time;
function Busy(ListenerValue: Net.Listener; Token: Tasks.CancellationToken): boolean;
begin
  const Client: Net.Connection := Net.Accept(ListenerValue).Unwrap();
  const Configured: boolean := Net.SetTimeout(Client, 1000).Unwrap();
  const Ignored: result of (array of integer, Net.NetError) := Net.ReceiveBytesWithCancellation(Client, 1, Token);
  return true;
end function;
function Reader(ListenerValue: Net.Listener; Token: Tasks.CancellationToken): boolean;
begin
  const Client: Net.Connection := Net.Accept(ListenerValue).Unwrap();
  const Configured: boolean := Net.SetTimeout(Client, 5000).Unwrap();
  case Net.ReceiveBytesWithCancellation(Client, 1, Token) of
    when Ok(const Bytes):
    begin
      return Bytes.Length() = 1;
    end;
    when Error(const Message):
    begin
      return false;
    end;
  end case;
end function;
function Quick(): integer;
begin
  return 7;
end function;
function Open(): Net.Listener;
begin
  return Net.Listen('127.0.0.1', 0).Unwrap();
end function;
function Join(ListenerValue: Net.Listener): Net.Connection;
begin
  return Net.Connect('127.0.0.1', Net.ListenerLocalAddress(ListenerValue).Unwrap().Port, 1000).Unwrap();
end function;
begin scope
  const Source: Tasks.CancellationSource := Tasks.CreateCancellationSource();
  const Token: Tasks.CancellationToken := Tasks.GetCancellationToken(Source);
  const FirstBusy: Net.Listener := Open();
  const SecondBusy: Net.Listener := Open();
  const ReaderListener: Net.Listener := Open();
  const FirstClient: Net.Connection := Join(FirstBusy);
  const SecondClient: Net.Connection := Join(SecondBusy);
  const ReaderClient: Net.Connection := Join(ReaderListener);
  const First: task of boolean := go Busy(FirstBusy, Token);
  const Second: task of boolean := go Busy(SecondBusy, Token);
  const ReaderTask: task of boolean := go Reader(ReaderListener, Token);
  const QuickTask: task of integer := go Quick();
  const Started: integer := Time.TimestampMillis();
  if Tasks.Wait(QuickTask) <> 7 then panic('quick'); end if;
  const Sent: integer := Net.SendBytes(ReaderClient, [42]).Unwrap();
  if not Tasks.Wait(ReaderTask) then panic('the reader did not receive the byte sent after the wait'); end if;
  if Time.TimestampMillis() - Started > 4000 then panic('the wait ran the reader inline'); end if;
  if not Tasks.Wait(First) then panic('first'); end if;
  if not Tasks.Wait(Second) then panic('second'); end if;
 end scope; end."#,
        2,
    );
}
