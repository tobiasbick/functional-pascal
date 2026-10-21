# Explicit local cycle reclamation

VM domain/session construction installs `LocalCollector` as an explicit-only
`DomainMaintenance` hook. A domain retains the stateless hook without retaining a
worker, callback registry or session. Internal callers request a pass through
`MemoryDomain::request_explicit_maintenance`; running execution hands the request to
AP48.5 safe points. Ordinary allocation, mutation and root release never activate
automatic collection. No FPAS collection command is exposed.

The root/exclusion prerequisites are the [root inventory](ROOTS.md),
[execution protocol](EXECUTION.md) and the shared-value cell-cycle proof in the
[runtime ownership handbook](../../../../../docs/pascal/program-structure/runtime-ownership.md).
The algorithm lives with the shared graph/cell operations in
`fpas-bytecode/src/value/collector/`; VM orchestration does not duplicate its walker
or depend on task-specific channels/callbacks.

## Preparation

1. Under exclusive domain ownership, snapshot the weak cell registry and retain
   live candidates. Candidate handles and all collector scratch are bookkeeping,
   never roots. Allocation/release counters and weak handles observe reclamation.
2. Reuse the iterative shared walker to mark registered execution/external roots,
   following every aggregate, callable capture/receiver, cell and cell-reference
   edge. Shared allocations are visited once. Foreign cells reject before their
   payload is read outside domain authority. Global references contribute owner
   identity; their actual storage is already registered separately.
3. Finish marking and reserve every candidate/index/lock/release buffer before
   changing cells. `CollectionLimits` bound candidate count and visited values;
   exceeding a bound or failing `try_reserve` returns `ScratchUnavailable`.
   Busy or poisoned providers/cells abort preparation. Unknown state stays live.
4. Acquire every unreachable candidate's payload lock with nonblocking access.
   Any failed lock releases the prepared guards without changing a payload.

These bounds limit whether a whole pass can be prepared. They do not implement
incremental marking or a pause-time bound. A successful pass covers the entire
domain; collection frequency does not bound its pause.

## Commit and release

With every payload lock held and release storage reserved, replace unreachable
payloads with a preallocated shared internal Unit value and move the old immutable
payload snapshots to the release queue. All observers remain excluded by the domain
permit. Release every cell lock before draining the removed snapshots through
existing iterative teardown. Candidate handles keep cells alive until every edge
has been disconnected; release them afterward and prune dead weak registrations.
No user code/finalizer or callback runs from value teardown.

A commit guard remains armed through edge removal, iterative release, candidate
release and registry pruning. Unexpected unwinding in that interval permanently
poisons the domain. Fresh execution returns an ownership error; infallible graph
access and parked execution resumption panic instead of reopening the partially
changed graph. Wait cleanup restores nested token accounting before rejecting
resumption. Preparation failures leave the domain
usable and pending maintenance unacknowledged. A process allocator abort also
cannot return to normal execution; teardown allocation is not recovered midway.

Reference counting still releases acyclic data immediately. Live values preserve
cell identity, aliases, callable provenance and aggregate copy-on-write semantics.
Explicit roots remain valid after worker/session destruction and can independently
retain cycles in a retired domain. A pass after release of root A can reclaim A's
cycle while B remains rooted; an edge-removing mutation of B can disconnect another
cycle without dropping B. Empty payloads are unchanged by repeated passes.

## Diagnostics and verification owners

`MemoryDomain::collection_stats` reports allocated/released cell storage, cleared
nonempty payloads, completed/aborted collector passes and current live weak entries.
Counters are monotonic and saturating. Administrative handles can delay storage
release after payload clearing, so cleared and released counts are separate.

Shared-value tests cover self/mutual closure cycles, records, arrays, dictionary
keys/values, enums, Option/Result, shared reachable subgraphs, all holder kinds,
references, retired independent roots/mutations, acyclic immediate release, bounded
scratch, busy/poisoned storage, incomplete views, foreign roots, repeated passes,
interrupted commit, poisoned cross-thread wait resumption and deep graphs on a
small native stack.

VM regressions exercise a self-recursive closure export after VM destruction, native
caller re-entry, published native waits, saved completion/continuation state, actual
inspection handles and owner/abandoned-attempt case cleanup with an independent
inspection root. Root publication and generation/exclusion races remain covered by
AP48.5 tests. Automatic pressure thresholds, controlled benchmarks and production
activation remain AP48.7.
