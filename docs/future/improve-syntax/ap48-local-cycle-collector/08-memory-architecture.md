# AP48.8: Replacement memory architecture

Package: [AP48](README.md). Status: planned; design direction recorded, no prototype.
Prerequisite: delivered AP48.1 through AP48.7 and the corrected access protocol.
Effort: large. Decisions: Q44 and Q45.

## Mandate and goal

This hobby project may replace its runtime, compiler internals, bytecode, Rust
embedding interfaces and language semantics without backward compatibility.
Syntax changes are valid design options; do not artificially constrain the
architecture to preserve current syntax. Before implementing a syntax change,
present its rationale, before/after examples, alternatives and affected consumers
to the user, obtain explicit approval and record it in the owning work package.
The general redesign mandate does not approve a specific syntax change.
Do not build compatibility adapters or retain two
production memory models merely to preserve old interfaces.

The goal is a smaller, understandable ownership and reclamation model. Faster
mutable captures are a hypothesis to test, not a promised outcome. Correct
reclamation, predictable alias behavior and comprehensible Rust safety arguments
matter before throughput. A simpler correct design can be worthwhile without a
large speedup, but measured regressions must be visible before adoption.

The immediate request records this plan and publishes it. It does not start the
runtime rewrite. AP48.8 is complete only when the concrete design work below is
done; writing this proposal does not complete the work package.

## Current behavior and reason for reconsidering it

FPAS combines reference-counted values, copy-on-write aggregates, domain-owned
mutable cells, registered roots, domain exclusion and separate cycle reclamation.
Mutable captures already cannot cross task boundaries. Parallel tasks can share
eligible values and resource handles under their existing contracts.

The corrected implementation counts external guards even when acquired by the
executing owner. Otherwise an external borrow can survive a safe point, a native
wait or execution itself without preserving exclusion. That correctness fix
increases some capture-access costs. Replacing it with another unchecked owner
shortcut would repeat the bug.

Investigate removing the overlapping lifetime mechanisms for local objects,
rather than assuming the existing representation is a permanent constraint.
The [runtime comparison](runtime-memory-comparison.md) explains the useful ideas:
Wren's small tracing heap, BEAM's process ownership, and QuickJS's contrasting
reference-counting design. None establishes a performance result for FPAS.

## Candidate architecture

| Area | Proposed direction | Required design evidence |
| --- | --- | --- |
| Local object heap | One heap per logical task or execution domain | Owner identity survives worker migration and nested calls |
| Local reclamation | Nonmoving, whole-domain mark-and-sweep | Every object edge and root is enumerated; unreachable cycles reclaim |
| Local references | Cheap tagged values or object handles | No refcount increment or per-cell mutex needed on proven local access |
| Execution | One active mutator per heap | Collection and external inspection cannot overlap an unprotected borrow |
| Task communication | Explicit copy, transfer or constrained sharing at runtime boundaries | Publication is atomic and cannot expose foreign local pointers |
| Host/debugger | Registered persistent handles plus scoped access | Retention keeps the heap alive; releasing roots permits reclamation |
| Native resources | Explicit resource ownership and synchronization | No managed/native cycle can bypass tracing and lifetime rules |

"Explicit" task communication does not imply new source syntax. Existing task
and channel operations can implement an unambiguous transfer rule internally.
AP48.9 selects that rule before broad implementation.

## Heap and value design

Classify every value kind: immediate scalars, strings, arrays, records, enums,
closures, cells, variable references, task/channel handles, native resources and
future FPAS2 data types. Each needs a storage owner, trace rule, copy rule and
destruction rule. Adding a type must not require a second ad hoc GC protocol.

Start the prototype with local closures, cells and the aggregate edges needed
to connect them. A partial prototype must root every boundary into old storage;
it is not evidence that the final runtime can safely ignore mixed graphs.

Nonmoving collection avoids relocating live objects, but does not make raw
pointers safe by itself. Select stable allocation or generation-checked handles;
document stale-reference prevention and free-slot reuse. Identify the small
unsafe boundary, if any, and its invariants. Do not make local object access
thread-safe by silently adding an atomic operation to every access.

Preserving array/record value semantics is separate from preserving today's COW
representation. Options include immutable backing storage, persistent structures,
eager copies or a proven uniqueness mechanism. Tracing alone does not tell us
whether an aggregate is uniquely referenced. AP48.8 must choose a coherent
candidate before AP48.10 measures aggregate behavior.

Shared immutable storage may retain reference counting if it has a simple,
acyclic ownership relationship with local heaps. It must never hold hidden strong
back-pointers into local heaps. If that restriction cannot represent a value,
copy it, classify it as local or redesign it; do not introduce an uncollected
cycle between heaps and shared storage.

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
The initial collector is neither concurrent, incremental nor generational; these
would add barriers and protocols without evidence that we need them.

## Work and acceptance

Inventory the current value modules and the VM's `domain/ROOTS.md`,
`domain/EXECUTION.md` and `domain/COLLECTION.md`. Produce a replacement ownership
diagram, value-kind matrix, root inventory and lifecycle state machine. Include
running, suspended, native-blocked, retired-with-handles and destroyed states,
with transitions for cancellation and worker migration.

Resolve heap granularity, object representation, COW replacement, allocation
safe points and external resource cleanup. Record alternatives rejected for
concrete complexity or semantic reasons. No need to explore a JIT, WASM,
generational collector or a new surface language in this package.

Acceptance: every proposed pointer edge has an owner and trace/lifetime rule;
every transition names its roots and exclusive-access condition; no local
capture operation depends on unrestricted external access. The design supports
the boundary contract in AP48.9 and a bounded AP48.10 prototype.

Likely implementation owners are `crates/fpas-bytecode/src/value/` and
`crates/fpas-vm/src/vm/domain/`. Before implementation, list exact files to create,
replace and delete after inspecting the checkout. Crate boundaries may change;
do not preserve a split that forces a circular dependency. Planned rules remain
here until implemented; the current handbook stays accurate for the shipped VM.
