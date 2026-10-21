# AP48.6: Local cycle reclamation

Package: [AP48](README.md). Status: complete. Effort: large.
Prerequisite: [AP48.5](05-safe-points.md), including the AP48.1 cycle proof
and AP48.4 exhaustive root inventory.

## Delivered contract

VM-created task, callback/session, helper and debugger-detachment domains install a
stateless local collector. AP48.5 exclusion and complete-root publication determine
when it can run. The internal `MemoryDomain::request_explicit_maintenance` interface
remains available; [AP48.7](07-activation.md) installs scheduled production policy
and owns automatic requests from allocation, mutation, periodic polls and lifecycle
boundaries. No FPAS collection command or new ownership syntax is introduced.

Each pass traces the entire owning domain:

1. Retain a stable candidate set from the weak cell registry. Candidate handles and
   collector buffers are administrative references, never program roots.
2. Mark registered execution and external roots through the shared iterative graph
   walker. Follow aggregates, callable captures/receivers, cells, cell references
   and ownership metadata; visit shared allocations once. Reject foreign cells
   before reading their payloads.
3. Complete marking, reserve required candidate/mark/commit/release storage and
   acquire every unreachable payload lock before modifying any cell. Scratch bounds,
   reservation failures and busy/poisoned/foreign state abort preparation without
   clearing cells or acknowledging pending maintenance. Unknown state remains live.
4. With candidates pinned and every payload lock held, replace unreachable payloads
   with one preallocated internal empty value and queue the removed snapshots.
   The exclusive domain permit prevents observation of this intermediate state.
5. Release every cell lock, drain snapshots through existing iterative teardown,
   release candidate handles and prune dead weak registrations.

Reference counting continues to release acyclic data immediately. Reachable cells
retain identity, aliases, callable provenance and copy-on-write behavior. Explicit
roots can independently retain cycles after execution/session retirement; releasing
one root or disconnecting an edge permits a later pass to collect only
that unreachable component. Repeated passes are idempotent.

An armed commit guard remains active through edge removal, release and registry
pruning. Unexpected unwinding permanently poisons the domain. Fresh execution and
fallible access reject poison; infallible graph access and parked-wait resumption
panic before reopening execution. Wait cleanup preserves nested token accounting.
Allocator abort cannot return to normal execution; iterative release allocation is
not recovered midway through commit.

Raw `OwnedCell` and `CellSnapshot` handles are not automatically registered roots.
Low-level callers hold domain access until registered publication and keep payloads
rooted across safe points. Sealed host/debugger APIs provide that discipline. The
maintenance-only reclamation interface requires the trusted hook's complete
prepared root view and rejects calls outside exclusive collection ownership.

## Implementation ownership

- `crates/fpas-bytecode/src/value/collector/{mod,commit,stats}.rs` owns candidate
  preparation, nonblocking all-lock preflight, destructive commit, poisoning and
  allocation/reclamation counters. `domain.rs`, `cell.rs` and value exports expose
  checked domain-local operations. Successful cell mutations notify the shared
  maintenance boundary; AP48.7 policy determines when they request a pass.
- `value/graph.rs` remains the single iterative edge walker for classification,
  inspection and marking, with fallible scratch preparation and visited-value bounds.
- `value/roots/{access,execution,lifecycle,maintenance,mod}.rs` owns exclusive
  servicing, complete fallible root snapshots, poison rejection and wait resumption.
- `crates/fpas-vm/src/vm/domain/collector.rs` installs policy and constructs domains
  without retaining a worker, VM or hosted registry. VM/session/callback/worker/task
  and debugger-detachment constructors use these factories. Host mutation delegates
  its request notification to the shared cell boundary.
- Focused shared-value collector fixtures, domain integration tests and existing
  continuation/inspection/supervision owners exercise the actual retained roots.
  The saved failure-inspection fixture retries deferred try-lock attempts while
  preserving its controlled barrier and catalog-lock assertions.

Algorithm and exclusion details live in the runtime
[collection inventory](../../../../crates/fpas-vm/src/vm/domain/COLLECTION.md),
[root inventory](../../../../crates/fpas-vm/src/vm/domain/ROOTS.md) and
[execution protocol](../../../../crates/fpas-vm/src/vm/domain/EXECUTION.md).

## Regression coverage

- Self-referential and mutually recursive callable cells; cycles through arrays,
  dictionary keys/values, records, enums, Option/Result and bound receivers.
- Shared reachable subgraphs, aliases and surviving payload contents; candidate
  references do not keep otherwise unreachable cycles marked.
- Host roots after VM destruction, actual suspended completion/continuation state,
  `var` references, nested callback temporaries, published native waits and debugger
  inspection handles. Release of each final root permits reclamation.
- Independent rooted cycles in a retired domain, including a mutation that
  disconnects a component while another root and its cell identity remain valid.
- Automatically closed owner cases and abandoned retry attempts, with an independent
  inspection root preserving their cycle until that root is released.
- Deep shared graphs and wrapper chains on a small native stack; immediate acyclic
  release; observable allocated/released/cleared counters and weak lifetime checks.
- Candidate/graph scratch limits, partial marking, busy/poisoned storage, incomplete
  providers and foreign roots all leave payloads unchanged and permit safe retry.
- Interrupted commit rejects fresh access and cross-thread parked-wait resumption;
  nested execution-token cleanup remains balanced.

The closure lifetime and runtime ownership handbook describe reclamation
and the raw access/root discipline. The compact language reference is confirmed;
syntax, task/resource transfers and source-level semantics remain unchanged.
Independent GPT-6.1 Sol review and repair review are complete. Applicable Rust,
Pascal, project and documentation checks are recorded in the delivery report.

## Scheduling and package acceptance

[AP48.7](07-activation.md) delivers automatic scheduling, the agreed initial thresholds,
CLI configuration and package acceptance. Controlled local measurement and later
threshold tuning are optional follow-up under Q43. Collection frequency does not
bound a full-domain pause; incremental collection would require a separate mutation
protocol. Cloud correctness runs establish no performance budget.
