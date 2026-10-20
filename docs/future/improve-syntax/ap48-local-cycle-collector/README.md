# AP48: Task ownership and a local cycle collector

Status: Planned. Effort: large, across runtime values, VM, sema and host tooling.
The user requested planning documentation only. No implementation is authorized.
Completion is tracked in the [central plan](../README.md#package-status).

## Goal

Close the gaps in task ownership and value transfers before collecting unreachable
closure cycles. Keep ordinary reference-counted release and parallel task execution.
Collection pauses only the owning logical task and coordinates access to its domain.
This is a garbage collector, not a global interpreter lock. It does not promise
lock-free execution or a fixed maximum collection pause.

## Current behavior

The [closure contract](../../../pascal/language/functions/closures.md#concurrency)
already prohibits mutable captures crossing task boundaries, including task results.
The current runtime enforces ownership on closure invocation, but does not enforce
recursive ownership checks at every transfer. Closure construction checks direct
cells and function captures, rather than the complete contained graph.

Cells use strong shared references and can participate in cycles with closures.
Dropping registers alone does not release such a cycle. The static audit also found
values retained outside registers by callbacks, selection, debugging and embedding.
These are source-inspection findings, not measured leak or concurrency results.

Source entry points:

- [Runtime values](../../../../crates/fpas-bytecode/src/value/mod.rs),
  [variable references](../../../../crates/fpas-bytecode/src/value/reference.rs)
  and [iterative release](../../../../crates/fpas-bytecode/src/value/drain.rs).
- [Closure cells](../../../../crates/fpas-vm/src/vm/calls/closures.rs),
  [saved task state](../../../../crates/fpas-vm/src/vm/tasks/state.rs),
  [task admission](../../../../crates/fpas-vm/src/vm/tasks/spawn.rs)
  and [result publication](../../../../crates/fpas-vm/src/vm/tasks/scheduler.rs).
- [Static capability propagation](../../../../crates/fpas-sema/src/check/expr/task_bound.rs),
  [host callbacks](../../../../crates/fpas-vm/src/vm/callback.rs)
  and [retained inspection values](../../../../crates/fpas-vm/src/vm/inspection/values/model.rs).

## Proposed contract

1. Each logical task owns a unique memory domain, identified across VM and session
   instances. Moving between worker threads preserves that domain. Synchronous
   callbacks use their caller's domain; a reusable callback session retains its
   root domain across invocations.
2. A cell belongs to exactly one domain. A local graph cannot contain foreign cells.
   Encapsulated cell operations and a shared graph walker enforce this rule.
3. Values without reachable local cells, task-bound callables or escaping references
   may be shared between tasks of the same VM, subject to resource-handle contracts.
   Session imports additionally require matching executable provenance for every
   contained callable. Data transferability alone does not grant code access.
   A `const` binding alone does not prove transferability.
4. Cross-task transfers reject local graphs before publication. There is no implicit
   deep copy, ownership transfer or new FPAS ownership syntax in this package.
5. Every live local value is reachable from an enumerated execution root or a
   registered external root. Host and debugger access obey the domain protocol.
6. A local collector traces those roots and disconnects unreachable cell contents.
   This algorithm requires proof that every permitted strong value cycle passes
   through a registered cell. If that proof fails, revise the design before enabling
   reclamation; do not silently collect only some classes of cycles.
7. Reference counting and copy-on-write behavior remain. Existing per-resource
   locks remain where needed. Collection holds no VM-wide execution lock.

Preserve argument evaluation order, closure aliasing, task completion and result
consumption rules, cancellation, structured cleanup and existing global-access
restrictions. Owner identities are runtime metadata, not serialized task numbers.

Excluded: concurrent or incremental collection, hard real-time pause guarantees,
cross-task mutable closure migration, a public GC unit, user finalizers, replacing
all shared values with a tracing heap, and unrelated language or scheduler redesign.

## Decisions and gates

| ID | Question | Planning decision and authorization | Affected work packages |
| --- | --- | --- | --- |
| Q37 | Which ownership and collection contract does AP48 use? | Domain isolation, checked transfers, registered roots and local tracing retained under delegated planning; implementation not authorized | AP48.1 through AP48.7 |
| Q38 | How long do unused selection cases retain local callbacks? | Selected under delegated planning: close cases at owner termination and abandoned attempt cleanup | AP48.2, AP48.4, AP48.5, AP48.7 |
| Q39 | Who collects retired domains after external roots disappear? | Selected under delegated planning: every root release requests maintenance; a quiescent domain is serviced synchronously by the releasing thread | AP48.4 through AP48.7 |
| Q40 | When can blocked tasks be collected? | Selected under delegated planning: saved tasks and explicitly published native waits are eligible; opaque native calls defer collection until return | AP48.4, AP48.5, AP48.7 |
| Q41 | Which callable values may enter another session? | Selected under delegated planning: require the identical shared executable instance as well as transferability and valid resource handles | AP48.1, AP48.2, AP48.4, AP48.7 |

Q38 through Q41 are concrete plan decisions requested by the user. They do not
authorize implementation or change the current handbook. Q37 records the overall
design; explicit implementation authorization is still required. The work packages
below own the detailed rules and acceptance.

Q38 intentionally changes the current VM-lifetime retention of unused cases. Close
them after owner cleanup, without running callbacks or closing underlying resources.
An abandoned supervised attempt closes its own cases before retry. The logical task
retains its domain across retries. Returning from a reusable callback invocation is
not session termination; session cases close at shutdown or destruction.

Q39 applies to non-final root releases and host mutations that disconnect values,
not only destruction of the last handle. A per-domain request generation and the
permit-release handshake prevent lost maintenance requests. No collector thread is
introduced. The releasing thread may pay the cost of a complete local collection.

Q40 distinguishes scheduler suspension from native blocking. Native code publishes
all roots and releases execution access before an eligible wait. Opaque native calls
retain execution access, so there is no prompt-reclamation guarantee during them.

Q41 uses an opaque runtime token owned by the shared verified executable. Cloning its
shared handle preserves identity; loading equal bytes into a separate executable
creates a different identity. Numeric function IDs, names and byte equality do not
authorize import. Accepted calls resolve globals in the target session, with its
ordinary ownership checks. No source-session global references are imported.

The choice proposed here is local root tracing followed by cell-edge removal,
rather than reference-count snapshots or a concurrent cycle algorithm.

Open gates:

- Obtain implementation authorization for Q37 through Q41 before changing code.
  Further semantic changes outside these recorded decisions need a new decision.
- AP48.1 must establish the cycle-through-cells invariant and the complete walker.
- AP48.4 must close raw host/debugger escape paths and specify external-root lifetime.
- AP48.5 must demonstrate exclusive collection and complete roots at safe points.
- AP48.7 must set workload-specific overhead and pause budgets from a controlled
  baseline before automatic collection is enabled. No timing claims exist yet.

## Dependencies and work packages

No prerequisite AP. Execute in the following order; each package includes its own
consumer migration and verification. All implementation work is not started.

- [ ] [AP48.1: Owned cells and graph classification](01-owned-values.md)
- [ ] [AP48.2: Runtime transfer enforcement](02-transfer-rules.md)
- [ ] [AP48.3: Static capability diagnostics](03-static-capabilities.md)
- [ ] [AP48.4: Execution and external roots](04-roots-and-embedding.md)
- [ ] [AP48.5: Safe points and domain lifecycle](05-safe-points.md)
- [ ] [AP48.6: Local cycle reclamation](06-cycle-collector.md)
- [ ] [AP48.7: Automatic scheduling and acceptance](07-activation.md)

This is the only work-package checklist. Finishing AP48.1 through AP48.4 establishes
the ownership milestone without enabling collection. AP48.5 establishes the
collection protocol. AP48.6 adds explicit reclamation. AP48.7 enables automatic runs.
Follow the [development process](../development-process.md) for delivery and status.

## Package acceptance

Unreachable cycles are reclaimed during long-running tasks and after task teardown.
Reachable cycles preserve identity and mutable state. All transfer variants reject
forbidden graphs, including values hidden behind aggregates or ordinary calls.
Host roots remain valid across collection and permit reclamation after any release,
even while another root survives after task termination. Unused cases do not retain
abandoned task or attempt graphs. Cross-session callables require matching code
provenance. Opaque native calls explicitly defer collection until execution returns.
Collecting one domain permits unrelated runnable tasks to make progress.

Acceptance evidence must distinguish live objects, allocator caches and process
resident memory. A falling process memory reading alone does not prove reclamation.
Use the acceptance cases in each work package and the
[shared constraints](../shared-constraints.md); reuse existing tests where adequate.

## Documentation and implementation ownership

Runtime value representation and graph traversal belong in `fpas-bytecode` value
modules. VM domain policy, root enumeration and collection scheduling belong in
focused `fpas-vm` modules. The former must not depend on the VM. Sema diagnostics
use the same semantic rules without duplicating runtime graph algorithms.

Update the closure, concurrency and variable-reference handbook pages with the
corresponding implementation, including the closure lifetime statement. Update host
Rust interface documentation when embedding changes. Confirm the compact language
reference, generated APIs and artifact behavior; no new syntax is planned.
This documentation change does not update current behavior under `docs/pascal/`.
