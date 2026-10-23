# AP49.1: Runtime event counters

Package: [AP49](README.md). Status: complete. Decision: Q54.

## Scope

Add `fpas_bytecode::profile`: an enumerated counter set, a process-wide enable
flag and striped atomic storage. A thread picks its stripe once, so counting from
many pool workers does not contend on one cache line. `enable()` resets all
counts; `snapshot()` sums the stripes. While disabled, `add` is one relaxed load.

## Counters

| Counter | Event site |
| --- | --- |
| `classify_calls`, `classify_visited` | Ownership classification walks and the allocations they visit |
| `collection_passes`, `collection_nanos`, `collection_visited` | Serviced maintenance passes, their duration and traced values |
| `safe_point_nanos` | Root publication and safe points after dispatch batches |
| `task_spawns`, `task_yields`, `task_parks` | Spawned tasks, time-slice yields and cooperative suspensions |
| `queue_contended`, `worker_idle_nanos` | Contended scheduler-queue locks and time pool workers wait for work |
| `cancellation_lookups` | Registry lookups by `IsCancellationRequested` (cache misses) |

Each counter has a stable snake-case name used by the report.

## Acceptance

Unit tests cover reset, disabled counting and multi-thread summation. Each event
site is exercised by an existing or new VM test with profiling enabled. Without
profiling, `cargo bench-fpas run --group vm` stays within normal noise.
