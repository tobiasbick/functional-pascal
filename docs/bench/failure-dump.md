# Failure-dump measurements

The `failure-dump` benchmark group compares the same compiled program with
capture disabled and `--failure-dump` enabled. Run it on a quiet local machine
with the same checkout and toolchain. Cloud runs validate the measurement path
only; no cloud timing is a performance baseline or evidence of overhead.

```text
cargo bench-fpas save d1-local --group failure-dump
cargo bench-fpas compare d1-local --group failure-dump
```

The harness builds the release CLI. Its
[FPAS driver](../../examples/pascal/failure-dump/failure-dump-benchmark.fpas)
builds each fixture once, warms both modes, then alternates their order over
nine measured pairs. Both modes execute the identical `.fpascp` in a fresh
ordinary CLI process. Both also pass the same unique report path: the enabled
mode as `--failure-dump PATH`, the disabled mode as an ignored program argument
after `--`. On Windows, a process whose command line was not seen recently
starts about 2–3 ms slower, so passing the path in only one mode would charge
that cost to capture. Checksums, expected diagnostics and report parsing reject
incorrect executions. There is no debugger or deterministic scheduler.

| Workload | Work exercised |
| --- | --- |
| `scalar` | One million checked integer additions |
| `calls` | 200,000 calls with a checked result |
| `globals` | One million updates to program-global storage |
| `spawn-wait` | 128 owned task starts/waits with checked arithmetic work |
| `suspension` | A worker's 32 real one-millisecond timer suspensions |
| `failure` | A small escaping panic and its report |
| `large-report` | Escaped NUL strings forcing report clipping near the 8 MiB bound |

The elapsed interval covers process launch, image admission, VM execution,
failure capture, writer startup, publication and ordinary shutdown. Failure
rows include serialization and filesystem writes/syncs. Compilation, warmup,
reading/parsing the published report and driver bookkeeping are outside the
timers. Suspension includes intentional waiting. These are CLI latency
measurements, not isolated instruction or scheduler microbenchmarks.

Each driver's stdout includes both sample arrays, their medians, the largest
report in bytes, and `100 * enabled_median / disabled_median - 100` percent.
The suite's `elapsed_ms` is the sum of both modes' measured samples; its
`throughput` column carries the within-run percentage. Inspect `raw_stdout` in
`.temp-data/bench/d1-local.json` for the individual samples. A disabled median
below the millisecond timer's resolution produces no percentage. Repeat noisy
results before drawing conclusions; compare equivalent workload arguments and
avoid other builds or tests during measurement. No zero-overhead claim follows
from an unchanged rounded median.

Generated projects, images and reports remain under
`.temp-data/bench/failure-dump-*` for inspection. The driver accepts an optional
new output directory after the workload and odd sample count; the regression
test uses its runner-owned scratch directory. The ASCII-only fixtures make
JSON character length equal UTF-8 byte length for the reported sizes.

The [runner smoke test](../../tests/runner/failure_dump_benchmark_test.fpas)
executes every workload with one measured pair and validates behavior. Its
timings, especially from debug builds or cloud machines, are not performance
evidence. No cloud measurements are recorded in benchmark history.

Independent lifecycle tests own queued/running/waiting/sleeping/terminal task
states, released snapshots, and a blocked native peer's shared collection
deadline. See [failure capture tests](../../crates/fpas-vm/src/vm/failure_capture/tests)
and the [CLI report tests](../../crates/fpas-cli/tests/failure_dump). The paired
CLI rows do not isolate live-peer collection from semantic owner cleanup, because
an escaping failure is selected only after its owned tasks are cleaned up. The
[live-peer tests](../../crates/fpas-vm/src/vm/failure_capture/tests/catalog/live_peers.rs)
therefore drive running pool tasks directly at the embedded VM observer. Their
ignored profiling test prints the time from the root failure to its delivered
observation:

```text
cargo test -p fpas-vm --release --lib live_peer_collection_profile -- --ignored --nocapture
```

The implemented JSON contract lives in
[runtime failure reports](../pascal/tools/failure-dump.md).

## Local measurements: 2026-10-15

Three complete runs used a working tree based on `c49c0ae20`, Rust 1.99.0 and
the release CLI on the same quiet local system. Each workload warmed both modes
and then alternated nine measured pairs, with the unique report path passed in
both modes. All checksums, diagnostics, report parsing and size checks passed.

```text
cargo bench-fpas save d1-balanced-2026-10-15 --group failure-dump
cargo bench-fpas save d1-balanced-2026-10-15-repeat --group failure-dump
cargo bench-fpas record "Failure dump balanced paired latency baseline" --group failure-dump
```

The two saved snapshots retain all raw sample arrays under `.temp-data/bench/`.
Medians are rounded to whole milliseconds; a one-millisecond difference is one
timer tick.

| Workload | First disabled/enabled median (ms) | Repeat disabled/enabled median (ms) | Enabled change across three runs |
| --- | ---: | ---: | ---: |
| `scalar` | 22 / 22 | 21 / 22 | 0 to +1 ms |
| `calls` | 21 / 20 | 20 / 21 | -1 to +1 ms |
| `globals` | 39 / 39 | 40 / 40 | 0 ms |
| `spawn-wait` | 54 / 56 | 54 / 56 | +2 ms (+3.7%) |
| `suspension` | 514 / 517 | 513 / 516 | +0.4% to +0.6% |
| `failure` | 11 / 13 | 11 / 13 | +2 ms (+18.2%) |
| `large-report` | 13 / 62 | 14 / 62 | +48 to +49 ms |

Successful scalar, call and global workloads show no cost beyond timer
resolution. In-process timing agrees: enabling capture adds about 0.2–0.3 ms
per run for the writer thread and session setup, independent of the
instruction count. The capture-enabled dispatch loop's request check runs once
per 256 instructions. Spawn/wait has a real opt-in cost of about 15–20 µs per
task for catalog admission and release. Suspension is dominated by intentional
waiting. Failure rows include report construction, writing and syncing; the
large report adds 48–49 ms for 8,383,537 bytes. Small reports reached 1,067
bytes. Both sizes are below the 8,388,608-byte limit.

### Earlier unbalanced runs

Four earlier runs passed the report path only to the enabled mode and reported
+2.6% to +15.8% on successful scalar, call, global and spawn/wait workloads.
Separate process-phase timing showed that this difference occurred before the
CLI's `main` started: any new command-line argument, including one unrelated to
capture, cost the same. Those runs are superseded and are not part of
benchmark history.

### Live-peer collection

The [live-peer profiling test](../../crates/fpas-vm/src/vm/failure_capture/tests/catalog/live_peers.rs)
starts running pool tasks in busy loops, waits until all of them execute, then
fails the root. It measures from `Vm::run` to the delivered observation and
checks that every peer answered with its live frame. Its 21-sample release
results, in two runs after the collector fix below:

| Running peers | Median | Maximum |
| ---: | ---: | ---: |
| 0 | 4.5–5.9 µs | 38–46 µs |
| 1 | 18–21 µs | 72–107 µs |
| 4 | 55–80 µs | 106–116 µs |
| 16 | 248–251 µs | 367–600 µs |

Collection grows roughly linearly with live peers and stays far below the
500 ms collection deadline. Before the fix, the same test showed occasional
maxima of 11–18 ms with four or more peers. A peer published its response and
notified the collector without synchronizing with the collector's wait, so a
response arriving between the collector's scan and its wait went unnoticed
until the 5 ms poll interval, which Windows rounds up to its timer tick. A
change counter, checked under the wait lock, now makes the collector rescan
instead of sleeping.

A blocked native peer is covered separately by the shared collection deadline
test. An earlier FPAS CLI probe with an owned worker blocked in
`Std.Proc.RunCapture` measured semantic owner cleanup rather than collection:
its reports appeared only after the native operation finished and contained
only the root task.

## Process-start cost on Windows

The driver starts each child through `Std.Proc.RunCapture`, so most of the
elapsed time is process launch. On Windows the process engine places every child
in a Job Object so that its whole process tree can be supervised and terminated.
`process-wrap` creates the child suspended, assigns it to the job and resumes it
by scanning a snapshot of all system threads. On a development machine this costs
about 30 ms per spawn (`spawn` 34 ms with the job, 4 ms without; a full
`RunCapture` 50 ms against 16 ms), independent of the VM and its memory
management. Rows in this group are therefore dominated by about 30 ms for each of
the 18 measured processes, and the cost grows with the number of threads on the
machine. This is an accepted trade-off for reliable process-tree ownership; do not
read it as workload or capture overhead. The VM was ruled out by running the same
image directly (23 ms) and by timing `spawn` separately.
