# AP48.10: Bounded tracing-heap prototype

Package: [AP48](README.md). Status: planned; no implementation or measurements.
Prerequisites: the AP48.8 Phase 0 gate selected the architecture work, Q50 is
decided, and the [AP48.8](08-memory-architecture.md) and
[AP48.9](09-task-and-host-boundaries.md) contracts suffice for the prototype.
Effort: large. Decision: Q49.

## Purpose and scope

Test whether a single-owner, nonmoving tracing heap makes captures and lifetime
management substantially simpler while preserving the chosen language behavior.
The baseline is the corrected runtime at the commit measured in Phase 0. Rebuild
and measure it alongside the candidate; recorded timings are not a substitute for
a fresh comparison. A microbenchmark of unchecked pointer loads is not sufficient
evidence to replace it.

Implement one vertical slice: closure creation, shared mutable cells, calls and
returns, recursive/cyclic environments, aggregate edges as selected by Q50, persistent host roots,
collection and domain teardown. Exercise two independent heaps and at least one
chosen cross-task transfer path. Cover suspension and worker migration as actual
ownership transitions, even if the first harness is small.

Keep the prototype isolated from normal execution until the evaluation gate.
A temporary test harness or development-only integration is acceptable; a shipped
legacy/new runtime switch is not a goal. Record any unimplemented boundary so
prototype results cannot be mistaken for full-runtime acceptance.

## Construction order

1. Implement object storage, stable identity and iterative edge tracing. Add live
   object and payload-byte accounting independent of the collector's mark count.
2. Implement scoped roots, persistent handles and exclusive heap access. Establish
   allocation safe points before implementing constructors that can allocate.
3. Implement mark-and-sweep, free-list/allocation reuse and safe external cleanup.
   Sweep must not depend on recursive destruction of arbitrarily deep graphs.
4. Implement shared closure cells and direct local reads/writes. Any remaining
   refcount or lock on this path must have a documented owner and purpose.
5. Connect call frames, aggregate storage, suspension, host retention and the
   selected transfer path. Prove that mixed old/new representations cannot hide
   untraced edges during the experiment.
6. Run correctness checks before measuring execution and memory. Describe the
   remaining integration costs before deciding whether to proceed.

## Correctness experiments

| Scenario | Required observation |
| --- | --- |
| Two closures share one counter | Updates remain visible through both aliases, including after frame return |
| Unreachable self/mutual cycles | All owned objects and payload buffers reclaim after forced maintenance |
| Rooted cyclic graph | State and identity survive repeated collections |
| Constructor allocation stress | Collection at every legal allocation point loses no temporary input |
| Deep graph | Marking, copying and destruction avoid Rust stack overflow |
| Saved task and worker migration | Heap identity and roots survive suspension and resumption elsewhere |
| Two executing domains | Collection in one does not impose a VM-wide execution stop |
| Transfer failure/cancellation | No partially visible result and no leaked destination graph |
| Native wait or reentrant callback | Live values remain rooted and execution access is correctly serialized |
| Retired task with host handles | Retained values survive; releasing roots reclaims newly unreachable graphs |
| Concurrent handle release/maintenance | No missed collection request, use-after-free or deadlock |
| Debugger inspection/edit | Access cannot overlap an unsafe execution or collection interval |

Port the observable intent of the three access regressions in
`crates/fpas-bytecode/src/value/roots/protocol_tests/lifecycle.rs`: collection
cannot invalidate live external access, parking cannot expose an active borrow,
and an external lifetime cannot lose protection merely because execution ended.
The exact old guard API need not survive. Use compile-time rejection tests where
the replacement makes invalid lifetimes unrepresentable, plus runtime tests for
remaining concurrent transitions.

## Measurement protocol

Store scripts, raw samples, logs, binaries/identifiers and reports only beneath
`.temp-data/ap48-tracing-prototype/`. Record compiler version, build profile,
commit/dirty patch identity, CPU policy, collector settings and benchmark sources.
Compare the baseline and candidate on the same machine; local runs are preferred
for timing, cloud runs supply correctness evidence. Never compare timings across
machines.

Measure mutable/real/Boolean/string captures, closure calls, array callbacks,
non-capturing calls, allocation churn, aggregate copies, transfers and tasks.
Include retained graphs and large payloads so a faster capture loop cannot hide
new copying costs. Separate VM execution from compilation/startup where possible.
If bytecode changes, compile the same sources for each runtime and compare their
outputs; do not force incompatible artifacts through an old format.

Use repeated interleaved baseline/candidate runs, warmup and a recorded comparison
formula. Report distributions or medians with spread, not just one run. Record
peak/live managed bytes, allocation volume and collection pauses/counts alongside
execution time. RSS includes allocator caches and is supporting information only.
An unchanged RSS does not prove a leak; a falling RSS does not prove correctness.

## Evaluation gate

The prototype can proceed to AP48.11 only with a complete safety argument for its
implemented paths, passing lifetime/reclamation tests and a written comparison.
Explain which mechanisms disappear and which costs move to allocation, collection
or transfer. List remaining boundary work rather than hiding it behind a benchmark.

Choose proceed, revise or abandon based on that evidence. No numerical speedup is
pre-approved. If the design is simpler but slower, state the tradeoff explicitly;
do not silently redefine the workload or tune away correctness checks. A failed
prototype is useful evidence and does not authorize shipping a partial collector.
A proceed decision includes splitting AP48.11 into separately reviewable work
packages.

Inspect module ownership before selecting exact prototype paths. Keep reusable
tests and design results in the repository; generated artifacts stay in
`.temp-data`. Update this file with actual coverage and the gate result.
