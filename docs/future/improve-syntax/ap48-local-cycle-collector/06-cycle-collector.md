# AP48.6: Local cycle reclamation

Package: [AP48](README.md). Status: not started. Effort: large.
Prerequisite: [AP48.5](05-safe-points.md) complete, including the AP48.1 cycle proof
and AP48.4 exhaustive root inventory. Reclamation stays disabled if either is missing.

## Work

Implement a full local trace at an exclusive safe point:

1. Capture a stable candidate set from the weak cell registry. Collector-owned
   temporary references are bookkeeping, not program roots.
2. Mark all cells reachable from execution and external roots through the shared
   iterative graph walker. Visit every relevant aggregate allocation once.
3. Finish marking before modifying any candidate. If traversal or required scratch
   storage cannot be completed, abort without clearing cells. Unknown state is live.
4. Retain the unreachable candidate cells while removing their payload edges. Replace
   each payload with an internal empty value and place removed contents in a release
   queue. No observer may see this intermediate state.
5. Release removed contents outside cell locks using the existing iterative teardown
   machinery. Drop collector temporaries and prune dead registration entries.

Reference counting continues to release acyclic values immediately. Keep value
identity, copy-on-write and alias relationships unchanged. Do not inspect a changing
global set of reference counts or weaken closure references to break cycles.

Collect an entire domain per pass. Bounding the frequency does not bound the pause
for a large reachable graph. Incremental marking would require a separate mutation
protocol and is excluded. Define failure handling for interrupted teardown; no
partially collected domain may resume as normal execution.

## Owners and migration

Add focused collector modules under the VM domain implementation. Reuse graph/cell
operations in `crates/fpas-bytecode/src/value/` and its iterative release helpers.
Keep the collector independent of task-specific callback/channel implementations.

## Acceptance and verification

- Reclaim self-referential closures, mutually recursive closure cells, cycles through
  records/arrays/dictionaries/enums and `Option`/`Result` wrappers, and cycles sharing
  reachable subgraphs. Verify both reclaimed cells and surviving values.
- Preserve a cycle reachable only from a host root, suspended continuation, `var`
  reference, callback temporary or debugger handle. Reclaim it after that root ends.
- Collector candidates and registries do not accidentally count as live roots.
- Deep graphs use bounded call-stack space. Incomplete marking never clears cells.
- Repeated explicit collection is idempotent. Acyclic release still occurs without
  waiting for collection. Teardown does not invoke user code while holding cell locks.

Use observable live-allocation/reclamation counters and existing weak-lifetime
checks where appropriate, not process memory alone. Keep diagnostic instrumentation
in the owning memory module; avoid unrelated production hooks solely for tests.
Deliver explicit collection through the internal domain interface first. Automatic
activation belongs to AP48.7. Update the closure lifetime documentation on delivery.
