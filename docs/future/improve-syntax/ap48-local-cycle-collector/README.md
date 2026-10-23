# AP48: Task ownership and memory management

Status: extension planned. Effort: very large, across runtime values, VM, sema
and host tooling. AP48.1 through AP48.7 are delivered, independently reviewed and
verified. AP48.8 through AP48.12 plan a replacement memory architecture; none is
implemented. The current runtime remains the corrected local cycle collector.
Completion is tracked in the [central plan](../README.md#package-status).

## Goal

Simplify ownership, local access and reclamation with a candidate single-owner
tracing heap per task/domain. Evaluate the design before replacing the delivered
reference-counted runtime. Parallel task execution remains a design goal; no fixed
pause or speedup is promised. See [AP48.8](08-memory-architecture.md) for the full
mandate, invariants and alternatives.

The user permits replacement of all internals, APIs, bytecode and semantics without
backward compatibility. Syntax changes are explicitly eligible design options;
the existing syntax must not artificially constrain the architecture. Before
implementing any FPAS syntax change, discuss its concrete proposal with the user
and obtain their explicit approval. Record the rationale, before/after examples,
affected consumers and approval in the owning work package. Broad redesign
authorization alone does not approve a particular syntax change.
This authorization removes compatibility constraints, but does not
choose every transfer or host-lifetime rule. AP48.8/AP48.9 resolve those details.
The present task records and publishes the plan; runtime implementation has not
started. Existing behavior below describes the delivered implementation, not a
requirement to preserve its representation in the extension.

## Current behavior

The [closure contract](../../../pascal/language/functions/closures.md#concurrency)
already prohibits mutable captures crossing task boundaries, including task results.
The runtime now uses unique logical-task domains, checked cells and a shared
iterative graph classifier. Closure construction, invocation and local storage
validate the complete contained ownership graph. Callable targets carry verified
executable provenance. Task and channel publication reject local graphs before
admission or mutation. Session imports check origin and callable code identity.
Unused selection cases follow task/attempt cleanup. Static capabilities distinguish
transferable, unknown and task-bound values; conservative flow joins retain known
local state. Unknown values keep the runtime guard. Execution and registered
external roots publish complete views under per-domain exclusion. Automatic local
collection reclaims unreachable cycles with per-domain allocation/mutation pressure
and periodic batch polls. Run/test/debugger commands share configurable thresholds.

Cells use strong shared references and can participate in cycles with closures.
Dropping registers alone does not release such a cycle through reference counting.
Collection traces registered roots and disconnects unreachable cell payloads.
Callback, selection, debugger and embedding roots preserve reachable values until
their ownership ends. Allocation counters and weak-lifetime regressions distinguish
reclaimed cycles from retained roots; no performance result is claimed.

Source entry points:

- [Runtime values](../../../../crates/fpas-bytecode/src/value/mod.rs),
  [variable references](../../../../crates/fpas-bytecode/src/value/reference.rs)
  and [iterative release](../../../../crates/fpas-bytecode/src/value/drain.rs).
- [Local collector](../../../../crates/fpas-vm/src/vm/domain/COLLECTION.md)
  and [registered roots](../../../../crates/fpas-vm/src/vm/domain/ROOTS.md).
- [Closure cells](../../../../crates/fpas-vm/src/vm/calls/closures.rs),
  [saved task state](../../../../crates/fpas-vm/src/vm/tasks/state.rs),
  [task admission](../../../../crates/fpas-vm/src/vm/tasks/spawn.rs)
  and [result publication](../../../../crates/fpas-vm/src/vm/tasks/scheduler.rs).
- [Static capability propagation](../../../../crates/fpas-sema/src/check/expr/task_bound.rs),
  [host callbacks](../../../../crates/fpas-vm/src/vm/callback.rs)
  and [retained inspection values](../../../../crates/fpas-vm/src/vm/inspection/values/model.rs).

## Delivered contract (AP48.1 through AP48.7)

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
| Q37 | Which ownership and collection contract does AP48 use? | Domain isolation, checked transfers, registered roots and local tracing approved for implementation | AP48.1 through AP48.7 |
| Q38 | How long do unused selection cases retain local callbacks? | Approved for implementation: close cases at owner termination and abandoned attempt cleanup | AP48.2, AP48.4, AP48.5, AP48.7 |
| Q39 | Who collects retired domains after external roots disappear? | Approved for implementation: every root release requests maintenance; a quiescent domain is serviced synchronously by the releasing thread | AP48.4 through AP48.7 |
| Q40 | When can blocked tasks be collected? | Approved for implementation: saved tasks and explicitly published native waits are eligible; opaque native calls defer collection until return | AP48.4, AP48.5, AP48.7 |
| Q41 | Which callable values may enter another session? | Approved for implementation: require the identical shared executable instance as well as transferability and valid resource handles | AP48.1, AP48.2, AP48.4, AP48.7 |
| Q42 | Which initial collection thresholds and public tuning controls are allowed? | Approved: 4096 cell allocations, 8192 successful cell mutations or 256 completed execution batches, whichever threshold is reached first; positive CLI overrides `--gc-cell-threshold=N`, `--gc-mutation-threshold=N`, `--gc-poll-interval=N` apply consistently to run/test/debugger execution and their domains | AP48.7 |
| Q43 | Are controlled performance budgets required before activation and package completion? | Approved: activate with the heuristic Q42 defaults after correctness acceptance, required checks, repaired independent review and documentation; later local measurement and tuning do not block AP48 completion | AP48.7 |

Q37 through Q43 are approved for implementation. The work packages below own
the detailed rules and acceptance. Update the current handbook with each delivered
contract, after verification and independent review.

Q38 replaces VM-lifetime retention of unused cases with owner cleanup, without
running callbacks or closing underlying resources.
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

The collector uses local root tracing followed by cell-edge removal,
rather than reference-count snapshots or a concurrent cycle algorithm.

Q42 permits CLI scheduling controls while keeping a public FPAS GC unit, manual
collection commands and a public disable mode outside this scope. Existing CLI
invocations and Rust embedding constructor signatures remain compatible. The
thresholds affect running collection frequency; they do not bypass safe points or
delay required retired/quiescent-domain maintenance.

Q43 completed activation acceptance for AP48.1 through AP48.7 without a fixed
performance or pause budget. Q44 through Q49 govern the newly planned extension
and may replace the delivered contract through a coherent redesign.

### Replacement architecture decisions

| ID | Question | Current decision or remaining choice | Affected work packages |
| --- | --- | --- | --- |
| Q44 | What may be replaced? | User-authorized: all internal representations, APIs, bytecode and semantics; no backward compatibility. Syntax changes are valid design options without an artificial preservation constraint, but each concrete syntax change requires discussion and explicit user approval before implementation. Record the proposal and approval in its owning work package. | AP48.8 through AP48.12 |
| Q45 | What architecture should be evaluated? | Preferred candidate: single-owner heap per logical task/domain, nonmoving mark-and-sweep, cheap local references. Resolve value representation, aggregate semantics/storage, safe points and native cleanup in AP48.8. | AP48.8, AP48.10, AP48.11 |
| Q46 | How do graphs cross task boundaries? | Open design choice: specify copy, proven ownership transfer and constrained immutable/resource sharing. Start from the existing prohibition on foreign local cells; do not assume mutable captures must become transferable. | AP48.9 through AP48.12 |
| Q47 | How do host/debugger references survive? | Preferred candidate: persistent rooted handles plus scoped exclusive access. Resolve cross-thread access, retired-heap maintenance, re-entry and shutdown before integration. | AP48.9 through AP48.12 |
| Q48 | Which collector controls remain meaningful? | Open design choice: choose allocation/live-byte policy after prototyping; existing cell/mutation flags and Rust constructors may be replaced without compatibility adapters. | AP48.11, AP48.12 |
| Q49 | What justifies adopting the replacement? | Require correct lifetime/reclamation, a simpler explained ownership model and measured execution/memory tradeoffs. Prototype gate before integration; no promised speedup or numerical budget. | AP48.10 through AP48.12 |

These are explicit design tasks, not completed implementations. Record selected
rules and their consequences in the owning files before changing consumers.
The broad redesign authorization is already given; do not repeatedly seek
compatibility approval for internal changes. The explicit user-approval gate for
FPAS syntax changes still applies. Escalate a genuine blocker or a
material unresolved product tradeoff rather than silently choosing contradictory
semantics. The current handbook remains unchanged until implementation delivers
the selected behavior.

## Dependencies and work packages

No prerequisite AP. AP48.1 through AP48.7 are complete, including consumer
migration, verification, independent review and original package acceptance.
AP48.8 through AP48.12 are the new ordered extension, using the delivered runtime
as the corrected baseline. The work packages own their contracts and evidence.

- [x] [AP48.1: Owned cells and graph classification](01-owned-values.md)
- [x] [AP48.2: Runtime transfer enforcement](02-transfer-rules.md)
- [x] [AP48.3: Static capability diagnostics](03-static-capabilities.md)
- [x] [AP48.4: Execution and external roots](04-roots-and-embedding.md)
- [x] [AP48.5: Safe points and domain lifecycle](05-safe-points.md)
- [x] [AP48.6: Local cycle reclamation](06-cycle-collector.md)
- [x] [AP48.7: Automatic scheduling and acceptance](07-activation.md)
- [ ] [AP48.8: Replacement memory architecture](08-memory-architecture.md)
- [ ] [AP48.9: Task transfer, captures and host boundaries](09-task-and-host-boundaries.md)
- [ ] [AP48.10: Bounded tracing-heap prototype](10-tracing-heap-prototype.md)
- [ ] [AP48.11: Replace runtime memory management and migrate consumers](11-runtime-replacement.md)
- [ ] [AP48.12: Memory correctness, performance and package acceptance](12-memory-acceptance.md)

This is the only work-package checklist. Finishing AP48.1 through AP48.4 establishes
the ownership milestone without enabling collection. AP48.5 establishes the
collection protocol. AP48.6 adds explicit reclamation. AP48.7 enables automatic runs.
Follow the [development process](../development-process.md) for delivery and status.

## Delivered acceptance (AP48.1 through AP48.7)

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

## Extension acceptance

[AP48.12](12-memory-acceptance.md) owns final acceptance for the replacement.
AP48.10 must first demonstrate the design with a bounded prototype. Full delivery
requires all value kinds, task boundaries, host/debugger lifetimes and retired
heaps to use a coherent model, removal of temporary bridges, applicable checks,
and a reported same-environment comparison. Reuse the safety intent of existing
tests while adapting obsolete APIs and explicitly changed semantics.

## Delivered documentation and implementation ownership

Runtime value representation and graph traversal belong in `fpas-bytecode` value
modules. VM domain policy, root enumeration and collection scheduling belong in
focused `fpas-vm` modules. The former must not depend on the VM. Sema diagnostics
use the same semantic rules without duplicating runtime graph algorithms.

Update the closure, concurrency and variable-reference handbook pages with the
corresponding implementation, including the closure lifetime statement. Update host
Rust interface documentation when embedding changes. Confirm the compact language
reference, generated APIs and artifact behavior; no new syntax is planned.
Benchmark artifacts belong under `.temp-data/`. Cloud runs provide correctness
evidence. Controlled local performance measurement and tuning are optional follow-up
under Q43; they do not block activation or package completion.

## Callback execution follow-up

[Array callback execution](array-callbacks.md) records the completed profiling,
inline callback dispatch and empty-collector path, including their rationale and
verification. All four steps are complete.

## Research and alternatives

[Runtime comparison](runtime-memory-comparison.md) records the QuickJS, BEAM and
Wren source review. [Execution access follow-up](execution-access-follow-up.md)
contains the Lua review and the narrower optimization alternative. The preferred
next direction is now the AP48.8 architecture investigation, rather than requiring
further patches to the existing memory model first. Neither candidate is implemented.
