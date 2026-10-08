//! Main-task channel and task waits: progress through the pool without running queued tasks inline.

fn run_with_workers(source: &str, workers: usize) {
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    let mut vm = crate::vm::Vm::new(fpas_compiler::compile(&program).expect("compile"));
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
uses Std.Net, Std.Tasks, Std.Time;
function BlockingRead(ListenerValue: Listener; Token: CancellationToken): boolean;
begin
  const Client: Connection := Accept(ListenerValue).Unwrap();
  const Configured: boolean := SetTimeout(Client, 1500).Unwrap();
  const Ignored: result of array of integer, string := ReceiveBytesWithCancellation(Client, 1, Token);
  const Closed: boolean := Close(Client).Unwrap();
  return true;
end function;
begin
  const Source: CancellationSource := CreateCancellationSource();
  const Token: CancellationToken := GetCancellationToken(Source);
  const FirstListener: Listener := Listen('127.0.0.1', 0).Unwrap();
  const SecondListener: Listener := Listen('127.0.0.1', 0).Unwrap();
  const FirstClient: Connection := Connect('127.0.0.1', ListenerLocalAddress(FirstListener).Unwrap().Port, 1000).Unwrap();
  const SecondClient: Connection := Connect('127.0.0.1', ListenerLocalAddress(SecondListener).Unwrap().Port, 1000).Unwrap();
  const First: task := go BlockingRead(FirstListener, Token);
  const Second: task := go BlockingRead(SecondListener, Token);
  const Events: channel of integer := CreateChannel(1);
  var Started: integer := TimestampMillis();
  const Outcome: result of integer, string := ReceiveWithTimeout(Events, 100);
  if TimestampMillis() - Started > 1000 then panic('timed receive ran a blocking task inline'); end if;
  if Outcome.IsOk() then panic('nothing was sent'); end if;
  Started := TimestampMillis();
  const Full: boolean := SendWithTimeout(Events, 1, 100).Unwrap();
  const Blocked: result of boolean, string := SendWithTimeout(Events, 2, 100);
  if TimestampMillis() - Started > 1000 then panic('timed send ran a blocking task inline'); end if;
  if Blocked.IsOk() then panic('the channel was full'); end if;
  if not Wait(First) then panic('first'); end if;
  if not Wait(Second) then panic('second'); end if;
end."#,
    );
}

/// Without inline helping, the pool worker still serves main-task producer/consumer waits.
#[test]
fn untimed_channel_waits_progress_through_the_pool_worker() {
    run_with_one_worker(
        r#"program PoolServesMainChannelWaits;
uses Std.Tasks;
function Doubler(Requests: channel of integer; Replies: channel of integer): integer;
begin
  var Count: integer := 0;
  for Index: integer := 1 to 50 do
  begin
    const Value: integer := Receive(Requests).Unwrap();
    const Sent: boolean := Send(Replies, Value * 2).Unwrap();
    Count := Count + 1;
  end; end for;
  return Count;
end function;
begin
  const Requests: channel of integer := CreateChannel(1);
  const Replies: channel of integer := CreateChannel(1);
  const Worker: task := go Doubler(Requests, Replies);
  for Index: integer := 1 to 50 do
  begin
    const Sent: boolean := Send(Requests, Index).Unwrap();
    if Receive(Replies).Unwrap() <> Index * 2 then panic('reply'); end if;
  end; end for;
  if Wait(Worker) <> 50 then panic('count'); end if;
end."#,
    );
}

/// While the main task waits for `Quick`, it must not run the queued `Reader`, which waits for a
/// byte the main task sends only after that wait returns. Two busy tasks keep both workers
/// occupied so `Reader` and `Quick` are still queued when the main task starts waiting.
#[test]
fn task_wait_does_not_run_a_queued_task_that_waits_for_the_main_task() {
    run_with_workers(
        r#"program TaskWaitDoesNotHelp;
uses Std.Net, Std.Tasks, Std.Time;
function Busy(ListenerValue: Listener; Token: CancellationToken): boolean;
begin
  const Client: Connection := Accept(ListenerValue).Unwrap();
  const Configured: boolean := SetTimeout(Client, 1000).Unwrap();
  const Ignored: result of array of integer, string := ReceiveBytesWithCancellation(Client, 1, Token);
  return true;
end function;
function Reader(ListenerValue: Listener; Token: CancellationToken): boolean;
begin
  const Client: Connection := Accept(ListenerValue).Unwrap();
  const Configured: boolean := SetTimeout(Client, 5000).Unwrap();
  case ReceiveBytesWithCancellation(Client, 1, Token) of
    when Ok(Bytes):
    begin
      return Bytes.Length() = 1;
    end;
    when Error(Message):
    begin
      return false;
    end;
  end case;
end function;
function Quick(): integer;
begin
  return 7;
end function;
function Open(): Listener;
begin
  return Listen('127.0.0.1', 0).Unwrap();
end function;
function Join(ListenerValue: Listener): Connection;
begin
  return Connect('127.0.0.1', ListenerLocalAddress(ListenerValue).Unwrap().Port, 1000).Unwrap();
end function;
begin
  const Source: CancellationSource := CreateCancellationSource();
  const Token: CancellationToken := GetCancellationToken(Source);
  const FirstBusy: Listener := Open();
  const SecondBusy: Listener := Open();
  const ReaderListener: Listener := Open();
  const FirstClient: Connection := Join(FirstBusy);
  const SecondClient: Connection := Join(SecondBusy);
  const ReaderClient: Connection := Join(ReaderListener);
  const First: task := go Busy(FirstBusy, Token);
  const Second: task := go Busy(SecondBusy, Token);
  const ReaderTask: task := go Reader(ReaderListener, Token);
  const QuickTask: task := go Quick();
  const Started: integer := TimestampMillis();
  if Wait(QuickTask) <> 7 then panic('quick'); end if;
  const Sent: integer := SendBytes(ReaderClient, [42]).Unwrap();
  if not Wait(ReaderTask) then panic('the reader did not receive the byte sent after the wait'); end if;
  if TimestampMillis() - Started > 4000 then panic('the wait ran the reader inline'); end if;
  if not Wait(First) then panic('first'); end if;
  if not Wait(Second) then panic('second'); end if;
end."#,
        2,
    );
}
