# GC memory regressions

Run the allocation tests and pool bounds with:

```sh
cargo test -p fpas-runtime --test gc_memory -- --nocapture
cargo test -p fpas-runtime --lib value::managed_heap
```

The integration test binary installs a process-wide `stats_alloc` wrapper around
the system allocator. Each case launches itself in a separate process with one
test thread, so unrelated tests cannot change its measurements. Allocations,
deallocations and reallocation growth/shrinkage on all runtime threads contribute
to the balance. The production allocator is unchanged.

Every measured round must return to one fixed baseline in both requested bytes
and outstanding allocation count. No per-round allowance or baseline reset can
hide cumulative growth. The fixtures also require substantial allocation traffic.
A control case retains 256 KiB allocated on another thread, grows and shrinks it,
and then checks that releasing it on a third thread restores the baseline.

## Coverage

| Case | Required behavior |
| --- | --- |
| Aggregate cycles | Two-cell cycles through arrays, dictionary keys/values, records, enums, Option, both Result variants, captures and bound receivers release their backing storage. Payloads include a 64 KiB string and an 8192-element array. Weak observations and allocated/released cell counts verify actual collection before thread exit. |
| VM | Large captured closure cycles and live aliases behave correctly with default and 1/1/1 collection thresholds. Both successful execution and runtime failure release memory. |
| Tasks | Concurrent child tasks release cycles on completion and on failure with scope cancellation. The balance includes pool/timer threads and their teardown. |
| Callback cycles | 32,000 self-referential closures created by `Map` callbacks, each with a private 2 KiB payload, keep sampled peak growth below 16 MiB. A control run with collection thresholds disabled must exceed that bound, so the check detects reclamation deferred to VM teardown. |
| Callbacks | A persistent session repeatedly exports aliases, invokes them and handles failed callbacks without memory growth. An independent closure remains alive; its last host root is released after session destruction. |
| Debugger | Repeated breakpoint stops, variable expansion and detached evaluation of a cyclic captured value have stable memory use. Disconnecting a stopped session releases its current payload and inspection/helper roots. |

Live-session measurements warm up for at least 64 rounds and require four
consecutive identical footprints within 256 rounds. They then check 32 more rounds
against the unchanged baseline while the session remains alive. This allows bounded
thread-local pools and registry capacities to fill before checking stability.

Completed-thread measurements initialize Rust's thread machinery with a no-op
thread, then check every FPAS round, including the first, after all spawned threads
have joined. Thread-local pools are destroyed at that point; their cached buffers
are part of the required release, rather than an allowance subtracted from the
balance. A child completion marker also ensures a mistaken test filter cannot
silently pass without executing the requested case.

The value pool tests separately saturate value, dictionary and string buffers
to exercise byte and bucket limits independently. They check take/recycle
accounting, oversized-buffer rejection, and the payload-box count limit. Recycled
payload boxes must contain only Unit and release a nested cell with a large string.

## Measurement boundary

These are balances of requested allocations through Rust's global allocator.
They do not measure allocator metadata, native thread stacks, external native
allocators or process resident memory. In particular, returning memory to the
system allocator does not require a decrease in RSS. Compiled images intentionally
retained by the host remain outside the execution balance.
