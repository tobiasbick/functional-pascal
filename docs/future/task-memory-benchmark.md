# Future: Task memory benchmark

## Goal

We should be able to reproduce the synthetic concurrency memory benchmark from [How Much Memory Do You Need to Run 1 Million Concurrent Tasks?](https://pkolaczk.github.io/memory-consumption-of-async/) in Functional Pascal:

1. Spawn **N** concurrent tasks (`go`).
2. Each task waits **10 seconds** (idle, no CPU work).
3. Block until all tasks finish (`WaitAll`).
4. Measure **peak RSS** externally while the process is alive.

Scale targets used in that article: **1**, **10k**, **100k**, and **1M** tasks. FPAS would compare against **Go goroutines** and similar green-thread models, not Rust `async`/`await` runtimes.

## Current state

**Nothing useful happens today.** The language surface looks sufficient on paper (`go`, `Std.Task.WaitAll`, `Std.Time.Sleep`), but a naïve port does not produce a meaningful benchmark:

- **`Std.Time.Sleep` blocks an OS worker thread** ([`docs/pascal/std/time.md`](../pascal/std/time.md)). The VM runs spawned tasks on a small pool (`max(1, available_parallelism − 1)` workers; see [`docs/pascal/08-concurrency.md`](../pascal/08-concurrency.md)). Most tasks sit in the ready queue while a handful of workers sleep on the host clock.
- **No cooperative timer wait** — there is no async-style sleep that suspends a task without holding a worker (no timer wheel + `Yield` integration).
- **No progress output** — a bench program that only spawns and waits appears hung for a long time during the spawn loop and again during `WaitAll`, with no stdout or metrics.
- **Per-task memory is unbounded in user code** — retaining **N** `task` handles in an `array of task` plus **N** `TaskState` queue entries and result-map slots is required for `WaitAll` today; we have not validated or optimized footprint at 100k–1M scale.
- **No curated examples or measurement notes** in the repository (a draft under `examples/` was removed until the runtime can support the benchmark honestly).

So we **cannot** yet claim FPAS numbers comparable to the blog post, and we should not ship example programs that imply we can.

## What we need

| Area | Requirement |
|------|-------------|
| **Scheduling** | Cooperative wait that does not pin a pool thread for the full sleep duration (timer + resume on the shared ready queue, or equivalent). |
| **Spawn path** | Fast bulk spawn and queue growth at 100k–1M tasks without pathological mutex contention or redundant allocations per `TaskState`. |
| **Handles** | Clarify whether `WaitAll` must retain **N** handles in FPAS source, or whether a barrier primitive (e.g. spawn-and-forget with a join counter) is needed for fair memory comparison. |
| **Examples** | Four fixed-scale programs (1k / 10k / 100k / 1M) under `examples/pascal/concurrency/`, commented with the blog link and **external** RSS measurement commands — excluded from CI smoke tests. |
| **Validation** | Run at least 1k and 10k locally; document observed peak RSS and wall time on a reference machine before publishing 100k / 1M expectations. |

## Related docs

- Language: [`docs/pascal/08-concurrency.md`](../pascal/08-concurrency.md), [`docs/pascal/std/task.md`](../pascal/std/task.md), [`docs/pascal/std/time.md`](../pascal/std/time.md)

## Sketch (not runnable yet)

```pascal
{ Target shape once cooperative sleep exists — do not add as an example until it works. }

program TaskMemory10k;

uses Std.Array, Std.Task, Std.Time;

const TaskCount: integer := 10_000;

procedure Worker();
begin
  Sleep(10_000)  { must become cooperative, not thread-blocking }
end;

begin
  mutable var Tasks: array of task := [];
  for I: integer := 1 to TaskCount do
    Push(Tasks, go Worker());
  WaitAll(Tasks)
end.
```
