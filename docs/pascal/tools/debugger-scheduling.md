# Seeded batch scheduling

`fpas debug app.fpasprj --schedule-seed 42` explores task ordering under a
controlled debug scheduler. It runs once and returns the ordinary
[batch result](debugger-batch.md), including the seed and scheduling coverage.

```sh
fpas debug app.fpasprj --schedule-seed 42
fpas debug app.fpasprj --schedule-seed 42 --script input.toml --print State
fpas debug app.fpascp --source-root . --schedule-seed 42
```

Omitting the flag keeps the existing round-robin debug scheduler and real task
clock. Normal `fpas run` continues to use its concurrent worker pool. Seeded
scheduling is batch-only; combining the flag with DAP or JSONL is an argument
error. It does not start recording or implement external-event replay.

## Repeating and exploring a run

Supply one explicit decimal seed from `0` through `18446744073709551615`.
Leading zeros are accepted and normalized. Signs, spaces, separators, overflow,
missing values and duplicate flags are rejected. The tool never chooses a seed
automatically. Arguments following `--` are program arguments.

The same verified executable, runtime build/platform, scheduling algorithm,
seed, arguments, sealed text input, breakpoints and deterministic bounds repeat
the controlled dispatch order and program outcome across fresh CLI processes.
A real watchdog deadline or external interruption can cut the run at a different
place. Changed code, compiler output, runtime builds or inputs invalidate that
repetition premise. A seed alone is not a complete bug report.

Try different seeds to explore channel delivery, task completion, cancellation
and timer ordering. Different seeds can produce the same order or outcome.
The tool does not exhaust schedules, detect every race or reproduce an
interleaving from the ordinary concurrent worker pool. Mutable globals still
belong to the main task; the seed does not permit workers to write them.

This example releases both producers through the same cancellation event so
their subsequent channel sends can compete:

```pascal id=seeded-scheduling-channel
program SeededChannel;

uses Std.Tasks as Tasks, Std.Time as Time, Std.Console as Console;

procedure Produce(
  Queue: channel of integer;
  Token: Tasks.CancellationToken;
  Value: integer
);
begin
  const Ready := Tasks.CancellationCase(Token, procedure() begin end procedure);
  discard Tasks.Select([Ready]);
  discard Tasks.Send(Queue, Value);
end procedure;

begin
  scope
    const Queue: channel of integer := Tasks.CreateChannel(1);
    const Source := Tasks.CreateCancellationSource();
    const Token := Tasks.GetCancellationToken(Source);
    discard go Produce(Queue, Token, 1);
    discard go Produce(Queue, Token, 2);
    Time.Sleep(1000);
    discard Tasks.Cancel(Source);
    Console.WriteLn(Tasks.Receive(Queue).Unwrap());
    Console.WriteLn(Tasks.Receive(Queue).Unwrap());
  end scope;
end.
```

## Dispatch and virtual time

The versioned policy is `seeded-rounds-v1`. At each round the driver admits
queued spawns, polls readiness in stable task-ID order, collects runnable task
IDs in ascending order and shuffles them. Each eligible ID receives one existing
driver dispatch turn. Newly admitted or awakened tasks absent from the list join
the next round; ineligible entries are skipped. Continuously runnable tasks
cannot lose all their turns to a randomly preferred peer.

Only shuffles consume the private SplitMix64 stream. Descending Fisher–Yates
uses unsigned 64-bit arithmetic and rejection sampling for each index. Neither
host entropy nor program `Std.Random` supplies scheduler randomness. A turn
uses the existing dispatch boundary; native work or synchronous callbacks inside
it are not newly preempted. No complete turn history is retained.

Virtual task time starts at zero milliseconds. When the driver next schedules
after consuming a nonempty round, it adds one millisecond. A stop or completion
can capture that final round before its closing tick. When no task is runnable,
the driver first settles readiness
and cleanup, then jumps to the nearest pending internal deadline. Timers also
progress while CPU work remains runnable. This is an artificial debug clock,
not an estimate of normal execution speed or CPU duration.

Sleep, timed channels, controlled waits, selection timers, task-group close
budgets and supervision backoff share that clock. Equal deadlines retain stable
task-ID observation order. `Select` still commits the first ready source case;
`WaitAny` retains its input-order and failure-precedence rules. Readiness polling
can commit suspended channel operations before dispatch. Its stable task-ID
priority is not randomized, so this mode does not explore every competing-waiter
order. Read-only inspection does not poll waiters, advance time or draw randomness.

If live tasks are blocked with no possible internal wake source, execution ends
with `schedule_deadlock`, exit 3, and their actual state. If virtual time cannot
advance without exhausting its range, it ends with `schedule_clock_limit`,
exit 3. Host invocation/debugger deadlines remain real safety bounds and never
advance task time.

## Controlled inputs and effects

Allowed operations are pure computation, task/channel/group/cancellation
operations, virtual `Sleep`, supplied `Std.Args`, prequeued
`Console.ReadText`/`ReadLn`, and captured `WriteText`/`WriteLn`. Use the existing
runner TOML script for text lines. Input is sealed before execution; host stdin
is never read. An exhausted queue that is still open reports `FP5011`;
sealing scheduled input does not signal EOF.

Key-input script events are setup errors in seeded mode, before initialization.
Normal batch still supports key scripts. Other console effects, wall-clock time
reads, randomness, cryptography, filesystem, environment, processes, network,
HTTP and server operations are rejected before invocation with `FP5026`, exit 2.
The diagnostic identifies the effect and actual source location. A task-group
failure cannot hide this rejection from the batch result.

Synchronous native callbacks inherit the effect guard, including nested calls.
Task intrinsics, `Sleep`, task scopes, spawning and yielding inside such callbacks
are rejected: they cannot park on the debug task driver. Move those operations
into ordinary task code or resumable `Select` callbacks. Pure callbacks work.
Test filesystem/input-mutation/screen helpers also reject; pure assertions work.
No rejected effect is captured, replayed or replaced with a fabricated result.

## Result and limits

Every batch result has a bounded `schedule` object. With no seed its mode is
`default`, clock is `realtime`, status is `not_requested`, and seed, algorithm
and counters are null. A seeded result has these fields:

| Field | Meaning |
| --- | --- |
| `mode` | `seeded`. |
| `seed` | Normalized decimal string; no JSON precision loss. |
| `algorithm` | `seeded-rounds-v1`. |
| `clock` | Requested `virtual` policy, even if setup fails. |
| `status` | `not_started`, `controlled`, `rejected`, `interrupted` or `unavailable`. |
| `reasons` | Explicit bounded coverage reasons, empty for an observed controlled run. |
| `rounds` | Begun rounds, including a partial final round, as a decimal string. |
| `dispatches` | Attempted driver turns as a decimal string, distinct from the packed-instruction counter. |
| `virtual_time_millis` | Observed virtual clock as a decimal string. |

Setup or binding failure is `not_started`. A forbidden effect is `rejected`
with reason `unsupported_effect`. `controlled` describes execution governed by
the policy through the reported stop, completion, runtime failure, deadlock or
deterministic bound; it does not imply success or a complete program run.

Real deadline interruption is `interrupted` with `host_deadline`; external
cancellation uses `external_cancel`. Unexpected worker loss is `unavailable`
with `worker_failure`; other missing state uses `state_unavailable`. Without
an authoritative published worker result, counters stay null rather than
inventing zero or reusing an earlier stop. The parent retains a valid requested
seed even when preparation blocks or the worker cannot publish.

The existing instruction, output, timeout, trace, snapshot and escaped JSON
ceilings and exit precedence still apply. The total report remains at most
8 MiB. Schedule metadata survives optional-state clipping. Scheduling retains
one round's task IDs and constant-size counters, alongside the existing task
catalog. There is no extra unbounded schedule log, automatic seed sweep,
interactive scheduling protocol, reverse execution or record/replay.
