# AP48.8: Replacement memory architecture

Package: [AP48](README.md). Status: deferred; see [AP48](README.md#deferred-extension).
Prerequisite: delivered AP48.1 through AP48.7, including the external-access
exclusion fix. Effort: large. Decisions: Q45, Q49 (Phase 0 gate) and Q50.

## Goal

A smaller, understandable ownership and reclamation model. Correct reclamation,
predictable alias behavior and comprehensible Rust safety arguments matter before
throughput. Faster mutable captures are a hypothesis to test. A simpler correct
design can be worthwhile without a speedup, but measured regressions must be
visible before adoption.

## Current behavior

FPAS combines reference-counted values, copy-on-write aggregates, domain-owned
mutable cells (`Arc` storage with a mutex and domain access checks), registered
roots, domain exclusion and separate cycle reclamation. Aggregates carry
constant-time ownership summaries (`value/ownership_summary.rs`). Mutable captures
cannot cross task boundaries; ownership-neutral values and resource handles can.

The corrected implementation counts external guards even when acquired by the
executing owner (`17bb3f119`). Otherwise an external borrow can survive a safe
point, a native wait or execution itself without preserving exclusion. Replacing
it with another unchecked owner shortcut would repeat the bug.

## Phase 0: measure the corrected baseline

The performance motivation is not yet quantified. [Benchmark history](../../../bench/history.md)
shows `mutable_capture` at 196 ms before AP48 (`76ac7d6df`), about 545 ms after
delivery and 234 ms on `ea5a9ee7d`; `closure_call` improved from 145 ms to 135 ms.
The exclusion fix raised it to about 342 ms; [permit reuse for cell operations](execution-access-follow-up.md)
brought it back to about 200 ms. These are single-machine readings without a profile,
so the remaining cost distribution is unknown.

1. On one machine, run `mutable_capture`, `closure_call`, `array_callbacks` and the
   `concurrency` group for the current branch and for `76ac7d6df`, with release
   builds, warmup and interleaved repetitions. Report medians and spread.
2. Profile `mutable_capture` and `array_callbacks` on the current branch. Attribute
   time to domain access checks, the cell mutex, reference-count traffic, ownership
   classification, collector polls and remaining dispatch.
3. Record the summary in this file. Raw samples, scripts and logs go under
   `.temp-data/ap48-phase0/`.

Collection policy (measured with `fpas run --profile`): the fixed poll collected
every 256 batches regardless of heap size. With 20,000 live closures, 144 passes
traced 5.6 million values and took 93 % of execution time. Thresholds now grow
with the last traced graph (Q42 amendment): the same run needs 6 passes and
46 ms instead of 456 ms. A replacement collector should keep this proportionality.

Gate, presented to the user with the numbers (no fixed threshold):

- Memory-protocol costs are a small share and the gap to pre-AP48 is small: the
  performance motivation is gone. Continue only if simplification alone justifies
  the effort, or stop the extension.
- Remaining domain access checks dominate, but cell storage does not: extend the
  delivered permit reuse to those paths first and measure again.
- Cell storage, reference counting or classification dominate: continue with Q50
  and the architecture below.

## Aggregate storage (Q50)

Decide this before the rest of the design; it sets the scope of AP48.9 through
AP48.11.

Shared storage must never hold hidden strong edges into a local heap, or an
uncollected cycle between heaps and shared storage becomes possible. Arrays,
records, dictionaries, enums and `Option`/`Result` payloads can contain closures,
and closures reference cells. Tracing alone also gives no uniqueness signal for
copy-on-write.

**Candidate A, hybrid (recommended first).** Only cells and the closure
environments that hold them live in the domain heap, as handles accessed under
the execution owner without a mutex or per-access guard. Strings and aggregates
stay reference-counted with copy-on-write. An aggregate containing a local handle
is local by its ownership summary, as today; neutral aggregates stay shareable.
Marking traverses reference-counted aggregates reachable from roots; sweeping
unmarked cells drops their payloads. Every strong cycle passes through a cell, so
this reclaims all cycles. Transfers keep "reject local, share neutral" without
graph copying. Reference-count traffic on aggregates and strings remains.

**Candidate B, complete local heap.** All compound values live in the domain heap
as tagged values without reference counts. Copy-on-write is replaced by eager
copies, persistent structures or a compiler-maintained uniqueness mechanism.
Task boundaries then need graph copying or a separate shared immutable heap
(AP48.9). This rewrites every aggregate operation, strings and host values.

Choose with the Phase 0 profile: if cell access and classification dominate, A
suffices; B is relevant only if aggregate reference counting is a measured cost.
Record the rejected candidate and its reason here.

## Candidate architecture

| Area | Proposed direction | Required design evidence |
| --- | --- | --- |
| Local object heap | One heap per logical task or execution domain | Owner identity survives worker migration and nested calls |
| Local reclamation | Nonmoving, whole-domain mark-and-sweep | Every object edge and root is enumerated; unreachable cycles reclaim |
| Local references | Cheap tagged values or object handles | No refcount increment or per-cell mutex on proven local access |
| Execution | One active mutator per heap | Collection and external inspection cannot overlap an unprotected borrow |
| Task communication | Rule selected in AP48.9 | Publication is atomic and cannot expose foreign local pointers |
| Host/debugger | Registered persistent handles plus scoped access | Retention keeps the heap alive; releasing roots permits reclamation |
| Native resources | Explicit resource ownership and synchronization | No managed/native cycle can bypass tracing and lifetime rules |

## Heap and value design

Classify every value kind: immediate scalars, strings, arrays, records, enums,
closures, cells, variable references, task/channel handles, native resources and
future FPAS2 data types. Each needs a storage owner, trace rule, copy rule and
destruction rule according to Q50. Adding a type must not require a second ad hoc
GC protocol.

Nonmoving collection avoids relocating live objects, but does not make raw
pointers safe by itself. Select stable allocation or generation-checked handles;
document stale-reference prevention and free-slot reuse. Identify the small
unsafe boundary, if any, and its invariants. Do not make local object access
thread-safe by silently adding an atomic operation to every access.

## Collector and execution invariants

1. A local managed reference is usable only in its owning live heap.
2. One execution owner mutates that heap at a time, independent of OS thread.
3. Every value surviving a collection point is reachable from a published root.
4. A borrowed local reference cannot escape, park, trigger collection or enter
   arbitrary host code unless the protocol first ends the borrow and roots values.
5. Root mutation and collection are serialized. Host handles are roots, not
   unrestricted permission to dereference objects from another thread.
6. The complete graph is marked iteratively before unreachable objects are
   swept. Deep graphs cannot overflow the Rust call stack.
7. Sweep releases external resources without re-entering a half-swept heap.
   Deferred cleanup and failure policy must be specified where needed.
8. Collecting one domain does not require stopping unrelated runnable domains.

Choose collection points for allocation slow paths, dispatch boundaries, saved
tasks, callbacks, native waits and retired heaps. If allocation can collect,
constructor inputs and partially built objects must already be rooted. If
allocation defers collection, define the permitted overshoot and failure policy.
The initial collector is neither concurrent, incremental nor generational.

## Work and acceptance

After the Phase 0 gate and Q50: inventory the current value modules and the VM's
`domain/ROOTS.md`, `domain/EXECUTION.md` and `domain/COLLECTION.md`. Produce a
replacement ownership diagram, value-kind matrix, root inventory and lifecycle
state machine. Include running, suspended, native-blocked, retired-with-handles and
destroyed states, with transitions for cancellation and worker migration. Record
alternatives rejected for concrete complexity or semantic reasons.

Acceptance: Phase 0 results and gate decision recorded; Q50 decided; every proposed
pointer edge has an owner and trace/lifetime rule; every transition names its roots
and exclusive-access condition; no local capture operation depends on unrestricted
external access. The design supports AP48.9 and a bounded AP48.10 prototype.

Likely owners are `crates/fpas-bytecode/src/value/` and `crates/fpas-vm/src/vm/domain/`.
List exact files to create, replace and delete before implementation. Crate
boundaries may change; do not preserve a split that forces a circular dependency.
The existing `value/managed_heap.rs` is a buffer pool, not a heap; rename it if a
real heap module is introduced.
