# AP48 follow-up: Array callback execution

Package: [AP48](README.md). Status: steps 1 through 3 complete; step 4 open.
The delivered AP48.1 through AP48.7 ownership and collection contracts still apply.

## Goal and sequence

Reduce repeated callback/root management and empty collection work in
`array_callbacks`, while preserving closure aliases, complete roots, domain
exclusion, failure inspection and debugger behavior. The work follows four steps:

| Step | Work | Status |
| --- | --- | --- |
| 1 | Profile callback execution, root handling and collection | Complete |
| 2 | Use inline bytecode callback continuations on the main task | Complete |
| 3 | Complete collection early when no live cell candidates exist | Complete |
| 4 | Reduce remaining value copies and repeated validation | Open |

## Step 1: Establish the cause

The unchanged workload performs Map, Filter and Reduce over 64 integers, giving
192 callbacks per iteration. Isolated instrumentation and CPU sampling showed
four temporary root registrations/removals, one caller snapshot and two execution
publications per callback, plus one state registration per higher-order operation.
Each callback also caused a completed empty collection, although this workload
allocated no cells. Temporary-root removal requests mandatory maintenance, so
allocation/mutation threshold tuning would not remove these passes.

This established the implementation targets: reuse execution roots for bytecode
callbacks, then avoid cell-marking work when the domain has no cells. Production
sources and GC thresholds stayed unchanged during diagnosis. Instrumented sources,
samples, counters and reproduction inputs remain in
`.temp-data/array-callbacks-step1/`.

## Step 2: Share inline callback execution

Bytecode higher-order calls now use the existing continuation/call-frame path on
both main and scheduled tasks. The shared dispatcher covers Array, Dict, Str,
Option and Result. Caller/callback frames, pending results and operation state
remain execution roots on their owning worker. Individual elements no longer
require native caller snapshots, cached-worker setup and temporary caller,
argument and result registrations.

Normal opcode dispatch calls `execute_intrinsic::<true>`. Immediate debugger
evaluation calls `execute_intrinsic::<false>` and completes synchronously, as do
direct native callback invocations. The main callback's task/sleep/scope restrictions
remain enforced through its synchronous callback boundary. Inline recursion follows
the debugger's frame-retention policy; failure tests retain precise caller/callback
locations and parameter values. Native fault/cache-contention coverage remains
separate because that route still owns a cached worker.

Reusing the established continuation path removes repeated ownership handoffs and
lets callback instructions consume the normal dispatch budget. Regressions cover
one-instruction dispatch, nested partial results, recursive captured-cell aliases
under a 1/1/1 collection schedule, immediate debugger results and failure frames.

Implementation owners:

- [Intrinsic dispatch](../../../../crates/fpas-vm/src/vm/execute/intrinsics.rs)
  and [opcodes](../../../../crates/fpas-vm/src/vm/dispatch/opcodes.rs).
- [Callback orchestration](../../../../crates/fpas-vm/src/vm/hosted/callbacks/mod.rs)
  and [continuations](../../../../crates/fpas-vm/src/vm/hosted/callbacks/continuation.rs).
- [Callback regressions](../../../../crates/fpas-vm/src/vm/tests/inline_callbacks.rs)
  and the [execution](../../../../crates/fpas-vm/src/vm/domain/EXECUTION.md)
  and [root](../../../../crates/fpas-vm/src/vm/domain/ROOTS.md) inventories.

## Step 3: Complete empty collection passes early

The collector prunes its weak registry and prepares live candidates under exclusive
domain ownership. If that candidate set is empty, it validates the complete prepared
root view, counts a successful pass and returns before marking cells or preparing
a commit. Every permitted strong value cycle passes through a registered cell;
with no live candidates, no such cycle can need disconnection.

A bare return would also suppress invalid-root errors. Empty passes therefore use
the shared graph walker with the existing conservative ownership summaries. Neutral
aggregates can skip subgraphs with no cells, functions or references. Foreign cells
still reject before payload access; callable and global-reference owners remain
checked. Finite graph bounds retain full value counting, and resource inspection
continues to observe every value through its existing walker mode.

Root-provider preparation, domain exclusion, pending requests, generation
acknowledgement, successful pressure reset and collection thresholds retain the
same maintenance protocol. Six regressions cover empty-to-populated transitions,
finite bounds, incomplete-provider retry, foreign callable/reference owners,
exclusive collection and poisoned foreign cells behind all strong edge kinds.

The [collector](../../../../crates/fpas-bytecode/src/value/collector/mod.rs),
[shared walker](../../../../crates/fpas-bytecode/src/value/graph.rs),
[empty-pass tests](../../../../crates/fpas-bytecode/src/value/collector/tests/empty.rs)
and [collector inventory](../../../../crates/fpas-vm/src/vm/domain/COLLECTION.md)
own this behavior.

## Step 4: Remaining copies and repeated validation — open

Start from the completed inline/empty-pass implementation and establish which
copies and checks still occur in the active callback path. The original profile
includes work already removed by steps 2 and 3.

Concrete inspection targets are array materialization in
[operation planning](../../../../crates/fpas-vm/src/vm/hosted/callbacks/plan.rs),
element/accumulator copies in
[operation state](../../../../crates/fpas-vm/src/vm/hosted/callbacks/operation.rs),
and callback handle cloning and call preparation in continuations. Retain shared
input storage or move values where their remaining owners and lifetimes permit it.
For repeated validation, identify the exact invariant and interval in which it
cannot change before consolidating checks. Dynamic arguments, results and mutable
captures still require their applicable ownership checks.

Preserve copy-on-write behavior, alias identity, evaluation order, callback effect
restrictions, executable provenance, roots across suspension and collection, and
error/debugger observations. A smaller clone/check count must follow from a clear
ownership or invariant argument. Reuse the callback/collector regressions and add
focused cases for any new lifetime or validation boundary.

After implementation, run the applicable Rust/Pascal tests, strict workspace
Clippy, formatting and project checks, then repeat the same VM benchmark group.
Store instrumentation, benchmark snapshots and differences under `.temp-data/`.
Step 4 has not been implemented by this delivery.

## Verification and measurement records

The final implementation passed 5305 Rust workspace tests, with zero failures and
three intentional ignores, plus 565 standalone Pascal tests with one skip.
Focused bytecode tests passed 141 cases; focused VM tests passed 908 cases with
two intentional ignores. Build, strict all-target/all-feature Clippy, Rust/Pascal
formatting, all 22 app/example projects and documentation links passed. The source
was frozen throughout the serial benchmark and full verification; standalone
Pascal staging followed the Rust CLI/LSP tests.

The same 23 VM benchmarks and arguments ran after steps 2 and 3. The array callback
checksum remained 208000000 for 50000 timed iterations and 9600000 callbacks.
Cloud elapsed-time snapshots and numerical differences remain under
`.temp-data/bench/`, `.temp-data/array-callbacks-step2/` and
`.temp-data/array-callbacks-step3/`, alongside source hashes, executable copies,
verification logs and reports. Benchmark history has no new timing entries.
