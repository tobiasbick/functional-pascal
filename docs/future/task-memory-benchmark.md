# Task memory benchmark

## Status

Implemented and validated on 2026-07-15. Functional Pascal can now reproduce the synthetic
concurrency benchmark from [How Much Memory Do You Need to Run 1 Million Concurrent
Tasks?](https://pkolaczk.github.io/memory-consumption-of-async/):

1. Spawn **N** retained tasks with `go`.
2. Each task waits **10 seconds** without occupying a pool worker.
3. Block until every task finishes with `WaitAll`.
4. Measure peak process memory externally.

The parameterized benchmark is
[`examples/pascal/concurrency/task_memory_benchmark.fpas`](../../examples/pascal/concurrency/task_memory_benchmark.fpas).
Use arguments `1000`, `10000`, `100000`, and `1000000` for the fixed scales. The original article
also measures the one-task runtime baseline; pass `1` when that data point is needed.

## Implemented runtime behavior

- `Std.Time.Sleep` on a spawned task saves the task state in a millisecond-bucketed timer queue and
  releases the pool worker immediately. One timer-driver thread returns due buckets to the shared
  FIFO ready queue. Main-task `Sleep` remains a blocking host wait.
- Empty operand and call stacks release spare allocation before a sleeping task is retained.
- Unit-valued task results use a compact completion representation instead of retaining a full
  `Value` per completion.
- `WaitAll` sorts and deduplicates handle ids once, then uses the retained-task completion count to
  wait for the missing group before rescanning. It no longer scans all **N** handles after every
  individual completion.
- Result completion no longer wakes unrelated workers waiting for ready tasks.
- Timer-ready work wakes a spawned `Wait`/`WaitAll` waiter, preventing single-worker starvation
  when a spawned parent waits for a sleeping child.

Regression coverage:

- [`crates/fpas-vm/src/tests/concurrency/cooperative_sleep.rs`](../../crates/fpas-vm/src/tests/concurrency/cooperative_sleep.rs)
  forces one pool worker and proves multiple sleeps overlap.
- [`tests/concurrency/cooperative_sleep_test.fpas`](../../tests/concurrency/cooperative_sleep_test.fpas)
  covers the compiled FPAS surface end to end.

## Reference measurements

Release runner built with `cargo build --release -p fpas-cli`. Measurements were taken on Windows
11 Pro 64-bit (10.0.26200), AMD Ryzen AI 9 HX PRO 370, 12 cores / 24 logical processors. A
PowerShell parent process sampled `WorkingSet64` and `PeakWorkingSet64` every 10 ms while the child
was alive.

| Tasks | Program elapsed | External wall time | Peak working set |
|------:|----------------:|-------------------:|-----------------:|
| 1,000 | 10,012 ms | 10,098 ms | 10.19 MiB |
| 10,000 | 10,076 ms | 10,109 ms | 12.84 MiB |
| 100,000 | 10,660 ms | 10,723 ms | 34.26 MiB |
| 1,000,000 | 16,608 ms | 16,703 ms | 251.71 MiB |

These values validate runtime behavior; they are not directly interchangeable with the article's
Linux RSS results. Windows working set and Linux RSS differ, and `fpas run` parses and compiles the
source in the measured process before executing the VM. Reproduce competing runtimes on the same
machine and measurement tool before publishing comparative claims.

## Measurement commands

Build once before measuring:

```text
cargo build --release -p fpas-cli
```

Linux example:

```text
/usr/bin/time -v target/release/fpas run examples/pascal/concurrency/task_memory_benchmark.fpas -- 1000000
```

On PowerShell, start `target/release/fpas.exe` with `Start-Process -PassThru`, sample
`WorkingSet64`/`PeakWorkingSet64` while `HasExited` is false, and record wall time with
`Diagnostics.Stopwatch`.

## Comparison boundary

This program intentionally retains an `array of task` and uses `WaitAll`. It is structurally
comparable to the article's Tokio, async-std, C#, Node.js, Python, Elixir, and thread/virtual-thread
variants, which also retain task or thread objects. The Go variant uses a compact `WaitGroup`
instead. FPAS would need a task-group/join-counter primitive for a direct WaitGroup-style memory
comparison; this benchmark must not be presented as that variant.

## Related docs

- [Concurrency overview](../pascal/language/concurrency/README.md)
- [Scheduling](../pascal/language/concurrency/scheduling.md)
- [`Std.Task`](../pascal/std/concurrency/task.md)
- [`Std.Time`](../pascal/std/host/time.md)
