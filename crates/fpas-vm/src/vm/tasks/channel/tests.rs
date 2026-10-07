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
uses Std.Net, Std.Results, Std.Tasks, Std.Time;
function BlockingRead(ListenerValue: Listener; Token: CancellationToken): boolean;
begin
  const Client: Connection := Std.Results.Unwrap(Accept(ListenerValue));
  const Configured: boolean := Std.Results.Unwrap(SetTimeout(Client, 1500));
  const Ignored: result of array of integer, string := ReceiveBytesWithCancellation(Client, 1, Token);
  const Closed: boolean := Std.Results.Unwrap(Close(Client));
  return true;
end function;
begin
  const Source: CancellationSource := CreateCancellationSource();
  const Token: CancellationToken := GetCancellationToken(Source);
  const FirstListener: Listener := Std.Results.Unwrap(Listen('127.0.0.1', 0));
  const SecondListener: Listener := Std.Results.Unwrap(Listen('127.0.0.1', 0));
  const FirstClient: Connection := Std.Results.Unwrap(Connect('127.0.0.1', Std.Results.Unwrap(ListenerLocalAddress(FirstListener)).Port, 1000));
  const SecondClient: Connection := Std.Results.Unwrap(Connect('127.0.0.1', Std.Results.Unwrap(ListenerLocalAddress(SecondListener)).Port, 1000));
  const First: task := go BlockingRead(FirstListener, Token);
  const Second: task := go BlockingRead(SecondListener, Token);
  const Events: channel of integer := CreateChannel(1);
  var Started: integer := TimestampMillis();
  const Outcome: result of integer, string := ReceiveWithTimeout(Events, 100);
  if TimestampMillis() - Started > 1000 then panic('timed receive ran a blocking task inline'); end if;
  if Std.Results.IsOk(Outcome) then panic('nothing was sent'); end if;
  Started := TimestampMillis();
  const Full: boolean := Std.Results.Unwrap(SendWithTimeout(Events, 1, 100));
  const Blocked: result of boolean, string := SendWithTimeout(Events, 2, 100);
  if TimestampMillis() - Started > 1000 then panic('timed send ran a blocking task inline'); end if;
  if Std.Results.IsOk(Blocked) then panic('the channel was full'); end if;
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
uses Std.Results, Std.Tasks;
function Doubler(Requests: channel of integer; Replies: channel of integer): integer;
begin
  var Count: integer := 0;
  for Index: integer := 1 to 50 do
  begin
    const Value: integer := Std.Results.Unwrap(Receive(Requests));
    const Sent: boolean := Std.Results.Unwrap(Send(Replies, Value * 2));
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
    const Sent: boolean := Std.Results.Unwrap(Send(Requests, Index));
    if Std.Results.Unwrap(Receive(Replies)) <> Index * 2 then panic('reply'); end if;
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
uses Std.Arrays, Std.Net, Std.Results, Std.Tasks, Std.Time;
function Busy(ListenerValue: Listener; Token: CancellationToken): boolean;
begin
  const Client: Connection := Std.Results.Unwrap(Accept(ListenerValue));
  const Configured: boolean := Std.Results.Unwrap(SetTimeout(Client, 1000));
  const Ignored: result of array of integer, string := ReceiveBytesWithCancellation(Client, 1, Token);
  return true;
end function;
function Reader(ListenerValue: Listener; Token: CancellationToken): boolean;
begin
  const Client: Connection := Std.Results.Unwrap(Accept(ListenerValue));
  const Configured: boolean := Std.Results.Unwrap(SetTimeout(Client, 5000));
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
  return Std.Results.Unwrap(Listen('127.0.0.1', 0));
end function;
function Join(ListenerValue: Listener): Connection;
begin
  return Std.Results.Unwrap(Connect('127.0.0.1', Std.Results.Unwrap(ListenerLocalAddress(ListenerValue)).Port, 1000));
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
  const Sent: integer := Std.Results.Unwrap(SendBytes(ReaderClient, [42]));
  if not Wait(ReaderTask) then panic('the reader did not receive the byte sent after the wait'); end if;
  if TimestampMillis() - Started > 4000 then panic('the wait ran the reader inline'); end if;
  if not Wait(First) then panic('first'); end if;
  if not Wait(Second) then panic('second'); end if;
end."#,
        2,
    );
}
