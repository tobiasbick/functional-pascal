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
uses Std.Net as Net; uses Std.Results as Results; uses Std.Tasks as Tasks; uses Std.Time as Time;
function Require of (T)(Outcome: Result of (T, string)): T;
begin
  case Outcome of
    when Result.Ok(const Value): return Value;
    when Result.Error(const Message): panic(Message);
  end case;
end function;

function BlockingRead(ListenerValue: Net.Listener; Token: Tasks.CancellationToken): boolean;
begin
  const Client: Net.Connection := Require(Net.Accept(ListenerValue));
  const Configured: boolean := Results.Unwrap(Net.SetTimeout(Client, 1500));
  const Ignored: result of (array of (integer), string) := Net.ReceiveBytesWithCancellation(Client, 1, Token);
  const Closed: boolean := Results.Unwrap(Net.Close(Client));
  return true;
end function;
begin
  const Source: Tasks.CancellationSource := Tasks.CreateCancellationSource();
  const Token: Tasks.CancellationToken := Tasks.GetCancellationToken(Source);
  const FirstListener: Net.Listener := Require(Net.Listen('127.0.0.1', 0));
  const SecondListener: Net.Listener := Require(Net.Listen('127.0.0.1', 0));
  const FirstClient: Net.Connection := Require(Net.Connect('127.0.0.1', Results.Unwrap(Net.ListenerLocalAddress(FirstListener)).Port, 1000));
  const SecondClient: Net.Connection := Require(Net.Connect('127.0.0.1', Results.Unwrap(Net.ListenerLocalAddress(SecondListener)).Port, 1000));
  const First: task := go BlockingRead(FirstListener, Token);
  const Second: task := go BlockingRead(SecondListener, Token);
  const Events: channel of (integer) := Tasks.CreateChannel(1);
   var Started: integer := Time.TimestampMillis();
  const Outcome: result of (integer, string) := Tasks.ReceiveWithTimeout(Events, 100);
  if Time.TimestampMillis() - Started > 1000 then panic('timed receive ran a blocking task inline'); end if;
  if Results.IsOk(Outcome) then panic('nothing was sent'); end if;
  Started := Time.TimestampMillis();
  const Full: boolean := Results.Unwrap(Tasks.SendWithTimeout(Events, 1, 100));
  const Blocked: result of (boolean, string) := Tasks.SendWithTimeout(Events, 2, 100);
  if Time.TimestampMillis() - Started > 1000 then panic('timed send ran a blocking task inline'); end if;
  if Results.IsOk(Blocked) then panic('the channel was full'); end if;
  if not Tasks.Wait(First) then panic('first'); end if;
  if not Tasks.Wait(Second) then panic('second'); end if;
end program;"#,
    );
}

/// Without inline helping, the pool worker still serves main-task producer/consumer waits.
#[test]
fn untimed_channel_waits_progress_through_the_pool_worker() {
    run_with_one_worker(
        r#"program PoolServesMainChannelWaits;
uses Std.Results as Results; uses Std.Tasks as Tasks;
function Doubler(Requests: channel of (integer); Replies: channel of (integer)): integer;
begin
   var Count: integer := 0;
  for Index: integer := 1 to 50 do
  begin
    const Value: integer := Results.Unwrap(Tasks.Receive(Requests));
    const Sent: boolean := Results.Unwrap(Tasks.Send(Replies, Value * 2));
    Count := Count + 1;
  end; end for;
  return Count;
end function;
begin
  const Requests: channel of (integer) := Tasks.CreateChannel(1);
  const Replies: channel of (integer) := Tasks.CreateChannel(1);
  const Worker: task := go Doubler(Requests, Replies);
  for Index: integer := 1 to 50 do
  begin
    const Sent: boolean := Results.Unwrap(Tasks.Send(Requests, Index));
    if Results.Unwrap(Tasks.Receive(Replies)) <> Index * 2 then panic('reply'); end if;
  end; end for;
  if Tasks.Wait(Worker) <> 50 then panic('count'); end if;
end program;"#,
    );
}

/// While the main task waits for `Quick`, it must not run the queued `Reader`, which waits for a
/// byte the main task sends only after that wait returns. Two busy tasks keep both workers
/// occupied so `Reader` and `Quick` are still queued when the main task starts waiting.
#[test]
fn task_wait_does_not_run_a_queued_task_that_waits_for_the_main_task() {
    run_with_workers(
        r#"program TaskWaitDoesNotHelp;

uses Std.Arrays as Arrays;
uses Std.Net as Net;
uses Std.Results as Results;
uses Std.Tasks as Tasks;
uses Std.Time as Time;

function Require of (T)(Outcome: Result of (T, string)): T;
begin
  case Outcome of
    when Result.Ok(const Value): return Value;
    when Result.Error(const Message): panic(Message);
  end case;
end function;

function Busy(ListenerValue: Net.Listener; Token: Tasks.CancellationToken): boolean;
begin
  const Client: Net.Connection := Require(Net.Accept(ListenerValue));
  const Configured: boolean := Results.Unwrap(Net.SetTimeout(Client, 1000));
  const Ignored: Result of (array of (integer), string) := Net.ReceiveBytesWithCancellation(Client, 1, Token);
  return true;
end function;

function Reader(ListenerValue: Net.Listener; Token: Tasks.CancellationToken): boolean;
begin
  const Client: Net.Connection := Require(Net.Accept(ListenerValue));
  const Configured: boolean := Results.Unwrap(Net.SetTimeout(Client, 5000));
  case Net.ReceiveBytesWithCancellation(Client, 1, Token) of
    when Result.Ok(const Bytes):
      begin
        return Arrays.Length(Bytes) = 1;
      end;
    when Result.Error(const Message):
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
  return Require(Net.Listen('127.0.0.1', 0));
end function;

function Join(ListenerValue: Net.Listener): Net.Connection;
begin
  return Require(Net.Connect('127.0.0.1', Results.Unwrap(Net.ListenerLocalAddress(ListenerValue))
                                                   .Port, 1000));
end function;

begin
  const Source: Tasks.CancellationSource := Tasks.CreateCancellationSource();
  const Token: Tasks.CancellationToken := Tasks.GetCancellationToken(Source);
  const FirstBusy: Net.Listener := Open();
  const SecondBusy: Net.Listener := Open();
  const ReaderListener: Net.Listener := Open();
  const FirstClient: Net.Connection := Join(FirstBusy);
  const SecondClient: Net.Connection := Join(SecondBusy);
  const ReaderClient: Net.Connection := Join(ReaderListener);
  const First: task := go Busy(FirstBusy, Token);
  const Second: task := go Busy(SecondBusy, Token);
  const ReaderTask: task := go Reader(ReaderListener, Token);
  const QuickTask: task := go Quick();
  const Started: integer := Time.TimestampMillis();
  if Tasks.Wait(QuickTask) <> 7 then
    panic('quick');
  end if;
  const Sent: integer := Results.Unwrap(Net.SendBytes(ReaderClient, [42]));
  if not Tasks.Wait(ReaderTask) then
    panic('the reader did not receive the byte sent after the wait');
  end if;

  if Time.TimestampMillis() - Started > 4000 then
    panic('the wait ran the reader inline');
  end if;

  if not Tasks.Wait(First) then
    panic('first');
  end if;

  if not Tasks.Wait(Second) then
    panic('second');
  end if;
end program;
"#,
        2,
    );
}
