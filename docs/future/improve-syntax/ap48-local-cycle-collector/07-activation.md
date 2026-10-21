# AP48.7: Automatic scheduling and acceptance

Package: [AP48](README.md). Status: in progress; implementation paused. Effort: medium to large.
Prerequisite: [AP48.6](06-cycle-collector.md) complete.

## Work

Request collection using per-domain cell allocation and mutation pressure, serviced
at the AP48.5 safe points. Include periodic checks while execution continues, before
parking, at termination and after external-root release. A graph can become garbage
when a register or handle drops without a new cell allocation; scheduling must cover
that case. Use the Q39 maintenance executor and Q40 blocking classifications from
AP48.5. There is no collector thread or global pause. Public CLI threshold overrides
are approved by Q42; no FPAS collection unit, manual collection command or source
syntax is introduced. Opaque native calls defer collection until return/unwind.

Pressure thresholds may defer ordinary running-task collections. They must not
suppress a requested retired/quiescent-domain pass after an external-root removal or
edge-removing mutation. Such requests use the existing synchronous executor as soon
as exclusive access and complete roots permit service.

## Approved defaults and CLI configuration

Q42 approves these heuristic initial defaults for each domain:

| Trigger | Default | CLI override |
| --- | --- | --- |
| Newly allocated cells | 4096 | `--gc-cell-threshold=N` |
| Successful cell mutations | 8192 | `--gc-mutation-threshold=N` |
| Completed VM execution batches | 256 | `--gc-poll-interval=N` |

The first reached threshold requests collection at the next complete exclusive safe
point. The poll interval counts completed execution batches, not milliseconds or
individual instructions; it covers garbage created by dropping values without new
cell allocations. Reset pressure after successful collection, while deferred or
incomplete passes retain pending work. Thresholds do not delay mandatory collection
requests at retirement, publication before parking or external-root disconnection.

All overrides take positive decimal integers representable by the shared runtime
configuration. Reject zero, negative, malformed and overflowing values with ordinary
CLI argument diagnostics. Each omitted option keeps its default. Existing invocations
remain valid and use automatic collection with the defaults after activation.

Use one effective configuration consistently across run, test and debugger execution,
including test subprocesses, task domains, reusable callback sessions and debugger
helper domains. Existing Rust embedding constructors retain their call signatures
and select defaults; expose configuration through an additive runtime interface.
Thresholds affect scheduling only: they cannot bypass domain exclusion, complete-root
publication, ownership validation or fail-closed interrupted teardown.

This approval covers the three threshold options. A public `--gc=off` mode and manual
collection commands are outside the agreed scope. Reference counting continues to
release acyclic values immediately; live identity, aliases and copy-on-write remain.

## Activation and package completion

Q43 approves automatic activation using these defaults after correctness acceptance,
required checks, independent review, confirmed-finding repairs and documentation.
Controlled throughput, memory and pause measurements are not prerequisites for
activation or completion. Do not invent performance results or describe the defaults
as measured optima. This replaces the former controlled-budget activation gate.

Completing this work package and the package acceptance closes AP48, since AP48.1
through AP48.6 are delivered. Later local measurements and threshold optimization are
independent improvements; they do not leave AP48 open or create another prerequisite.

## Acceptance workloads

| Workload | Pass criterion |
| --- | --- |
| Long-running loop creates and drops closure cycles | Repeated runs return live local cells to a stable envelope after collection |
| Live mutable closures survive repeated passes | Results, alias identity and state match execution without collection |
| Deep/wide graphs and many immutable transfers | No stack overflow; reachable values, sharing and transfer behavior remain correct |
| Migration, channel waits, cancellation and retries | No deadlock, stranded task or retained abandoned domain |
| Root A released after task/VM destruction while independent root B survives | A's cycle is collected without waiting for B; B remains usable for inspection |
| External mutation disconnects a cycle without dropping the root handle | Retired-domain maintenance collects the disconnected cycle |
| Root release races with collection or permit exit | Every request is serviced; no self-deadlock or lost wakeup |
| Owner termination and supervised retry with unused Select cases | Old case roots disappear; cycles collect; callbacks/resources are not invoked or closed |
| Saved task and published native wait | Maintenance proceeds without dispatching the task or ending the native wait |
| Polling and opaque native waits | Polling services requests; opaque calls defer them and service them on return/unwind |
| Equal function IDs in distinct executable instances | Direct, nested and bound callables are rejected before target execution |
| Two sessions share one verified executable | Transferable functions run against target globals; foreign cells/handles remain rejected |
| Busy collector and independent worker | Independent worker progresses during collection |
| Callback/session/debugger churn | No accumulating detached cycles, stale roots or owner collisions |
| Allocation, mutation and periodic thresholds | Each trigger independently requests a safe pass; the periodic trigger works without new allocations |
| CLI defaults, individual and combined overrides | Effective values reach every execution domain and subprocess without changing program results |
| Invalid CLI thresholds | Zero, negative, malformed and overflowing values fail before execution with normal argument diagnostics |

Use live-allocation/reclamation counters and weak-lifetime observations to distinguish
collected cycles from pooled capacity and process resident memory. RSS alone does not
prove reclamation. No hard real-time or maximum-pause guarantee is introduced.

Optional later controlled measurements may compare the original baseline,
ownership enforcement alone and automatic collection, separating graph-checking
costs from collection costs. Throughput, live cells, pause distributions and root
release latency can guide tuning. Cloud runs provide correctness evidence only;
all experiment output belongs under `.temp-data/`.

## Owners, documentation and verification

Integrate scheduling in domain counters, dispatch and lifecycle code. Reuse existing
runtime benchmark infrastructure and closure/concurrency regression owners. Add only
workloads that distinguish a credible failure or cost; no implementation-mirroring
tests or synthetic public language APIs.

Complete all applicable checks from the [development process](../development-process.md#required-checks),
including Rust formatting/build/workspace tests, the FPAS suite and affected projects.
Check portable/cached artifacts, embedding consumers and debugging behavior.
Review the result against the contract and repository standards.

Update the closure, task, Select-lifetime and variable-reference documentation, Rust
embedding contracts, CLI help and runtime/tool documentation; confirm the compact
reference. Document the defaults, units and scope of the three overrides. Explain
reference counting plus local cycle collection and the absence of a global execution
lock. Completion requires passing package acceptance and required checks, repaired
independent review and current documentation. Performance tuning remains follow-up.

## Implementation scope

The explicit collector and complete root protocol are delivered. Add pressure-based
scheduling and CLI configuration without changing value, transfer, resource or task
semantics. Activate automatic collection in production VM factories with the approved
defaults as part of this delivery. Implementation is paused; this plan update does
not authorize resuming code changes.

- Add `crates/fpas-bytecode/src/value/roots/scheduling.rs` and focused scheduling
  fixtures for the approved defaults and nonzero threshold overrides. Integrate
  state and policy in `roots/{access,maintenance,mod,execution}.rs`; reset pressure
  only after successful service. Root release and quiescent mutation bypass running
  thresholds; native published parking requests a pass.
- Update `value/domain.rs` and `value/cell.rs` to record allocation/mutation pressure,
  preserving publication-before-service and all-lock release boundaries. Export the
  shared scheduling configuration through value/lib interfaces, with no FPAS source API.
- Connect saved-task publication in `crates/fpas-vm/src/vm/tasks/state.rs` to forced
  parking requests. Existing dispatch and native safe points provide periodic polls.
  Update `vm/domain/collector.rs` construction and VM/session/task configuration
  propagation to use automatic policy. Add focused automatic correctness regressions
  under `vm/domain/collector/` using both defaults and smaller test schedules.
- Extend `crates/fpas-cli/src/cli_input/` parsing, option types and help for the three
  overrides. Propagate effective configuration through run/test/debugger owners and
  test subprocesses; add argument-validation and end-to-end propagation regressions.
- Verify long-running garbage-only drop loops, reachable state/aliases, retired
  independent roots and mutation, parking, opaque calls, independent workers and
  host/session/debugger lifetime against real collection. Reuse previous acceptance
  coverage for task migration, cancellation, retries and executable provenance.
- Document scheduling, default values, CLI overrides and package acceptance. Update
  runtime inventories and confirm the compact reference; no language syntax change.

One GPT-6.1 Sol reviewer examines the implementation after solo coding. Fix confirmed
findings, complete applicable checks and update status. Keep this checkbox and AP48
open until automatic activation, CLI propagation, correctness acceptance, review and
documentation are delivered. After completion, wait without committing or integrating
unless separately authorized.
