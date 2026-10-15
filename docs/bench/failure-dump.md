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
ordinary CLI process. Checksums, expected diagnostics and report parsing reject
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
CLI rows do not isolate live-peer collection from semantic owner cleanup;
local profiling of that boundary remains part of D1 performance validation.

The implemented JSON contract lives in
[runtime failure reports](../pascal/tools/failure-dump.md).
