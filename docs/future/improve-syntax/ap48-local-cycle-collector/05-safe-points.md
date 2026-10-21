# AP48.5: Safe points and domain lifecycle

Package: [AP48](README.md). Status: complete. Effort: large.
Prerequisite: [AP48.4](04-roots-and-embedding.md) complete.

## Delivered contract

Each domain has one gate for execution, external access and exclusive maintenance.
A logical `ExecutionRoots` owner moves with saved task state; thread-bound execution
access is acquired at batch and lifecycle boundaries, without a global interpreter
lock or a mutex on every instruction. Reentrant same-thread callbacks share gate
ownership. Retired domains reject execution and keep guarded external graph access.

Completed dispatch, suspension and finalization boundaries publish initialized
registers, globals, pending completions, continuations, supervision and native roots.
The last returned value remains rooted through permit release, worker destruction
and host export. Host argument admission registers same-domain handoffs before
consuming incoming exports. Incomplete providers and register bitmaps defer work.

Native callback re-entry publishes every enclosing caller and registers temporary
operation values. A native wait releases execution only after a complete root view
is prepared; it reacquires execution before using values or returning to FPAS.
An enclosing external borrow or unavailable provider keeps the call opaque.

Unused owner cases close before terminal execution retirement. Normal completion,
failure, cancellation and completed structured cleanup release dead frames and
callback state. Recoverable debugger failures retain inspectable frames. Retry
closes abandoned cases and replaces attempt frames while preserving logical domain
identity and legitimate external roots. Reusable callback sessions retain a session
lease between invocations; shutdown/destruction retires it. VM globals remain
registered until VM destruction; independent host/debugger roots can outlive it.

## Maintenance execution

A per-domain request generation and dirty flag record external-root releases and
edge-removing mutations, including independent non-final releases. Generation
saturation does not hide new requests. Quiescent requesters try synchronous service;
running execution defers to a completed safe point. Idle callback sessions and
retired domains require no worker, original VM, global lock or collector thread.

Request recording, final acknowledgement and permit release share the gate mutex.
An earlier request stays with the owner; a later request observes a quiescent domain
and services itself. A complete pass repeats for requests or removals arriving during
maintenance. Busy/poisoned preparation and unwinding hooks retain pending work.
Nested access cannot recursively collect or wait on its own execution permit.

Same-thread root removal under external access stays registered until outer release.
Another thread may commit removal during opaque execution without waiting; an
in-flight removal counter excludes maintenance until value release completes.
During maintenance the removal queues and remains a root until committed by the
exclusive owner. Registry/cell locks are released before destructors or hooks run.

The gate mutex protects short ownership transitions only. Published root preparation
uses storage protocols; maintenance holds no scheduler, channel or task-catalog
locks. Scheduler/catalog scopes defer servicing until their storage locks are gone.
Saved-state diagnostic inspection discovers domain authority before taking access,
then claims storage without waiting. It restores state and releases catalog locks
before deferred servicing. Global/frame failure capture also owns domain access.
Stopped debugger mutation, trace installation and live-image replacement cover peer
frames and main-domain globals with stable task-order access.

The VM installs no production maintenance hook. Explicit maintenance and an optional
automatic policy provide the route for AP48.6/AP48.7; this package reclaims no cells
and does not activate automatic production collection.

## Blocking states

| State | Implemented rule |
| --- | --- |
| Scheduler-suspended task | Complete roots are published before queue/timer storage; readiness may be recorded during maintenance, but dispatch waits for ownership |
| Published native wait | Registered native temporaries and all enclosing execution providers are complete; execution parks and requesters may maintain the domain |
| Main-task polling channel/task/selection wait | Each blocking probe releases execution through publication, without running queued tasks inline |
| Opaque native call, failure cleanup or teardown | Execution stays owned; pending work is serviced after return/unwind before another FPAS batch |

The complete native entry-point classification, lock order and lifecycle ownership
are in [EXECUTION.md](../../../../crates/fpas-vm/src/vm/domain/EXECUTION.md).
Registering selected native values alone never authorizes collection of a blocked
call. No prompt-reclamation guarantee applies to opaque operations.

## Implementation owners

- `fpas-bytecode/src/value/roots/`: the domain gate, execution publication,
  sessions/retirement, weak registrations, request handshake and maintenance deferral;
  `value/cell.rs` exposes retired-domain errors and public exports expose the protocol.
- `fpas-vm/src/vm/domain/`, `worker.rs`, `worker/run.rs`, `dispatch.rs` and
  `execute/intrinsics.rs`: execution/native access, result roots and safe points.
- `vm/tasks/{state,pool,scheduler}.rs`, suspension and supervision modules: logical
  owner save/restore, migration, retry, terminal cleanup and scheduler deferral.
- `vm/{mod,run,callback,callback_call}.rs`, `host_value/` and debugger call sandboxes:
  session leases, argument/result handoffs and callback publication.
- `vm/tasks/channel/`, selection polling, scope waits and group close: explicit
  native-wait publication, with opaque fallback for unconverted native paths.
- `vm/debug/tasks/driver/`, debugger session mutation/live-image modules and
  `vm/failure_capture/`: guarded inspection/transition, saved-state restoration and
  retirement while explicit inspection roots remain independently valid.

Affected debugger protocol fixtures in `fpas-debug/tests/{jsonl,dap_quiescence}.rs`
retain events returned with the launch/configuration response as well as later
waits; asynchronous arrival must not discard the first stop or logpoint.

The public embedding contract is documented in
[runtime ownership](../../../pascal/program-structure/runtime-ownership.md).
The compact language reference is confirmed: this package changes no FPAS syntax
or language-level transfer rule. The [root inventory](../../../../crates/fpas-vm/src/vm/domain/ROOTS.md)
and native inventory describe the actual consumer boundaries.

## Regression coverage

Deterministic shared-bytecode protocol tests demonstrate execution/access exclusion,
complete and unavailable views, nested release, manual/automatic policy separation,
shared-lock deferral, published/opaque waits, retired independent roots, session
leases, migration, re-entry rejection, request races, generation saturation and panic
unwind. Collection in one domain leaves an unrelated domain runnable.

VM integration covers main-channel servicing before a peer sends, opaque-return
servicing, final callable export, nested callback roots and saved-task claim/resume
exactly once after maintenance. Existing task/supervision tests cover retry and
structured cleanup. Debugger regressions cover cancelled peer frame/case retirement,
blocked live-image graph validation and retained inspection semantics. Failure-capture
regressions cover nonblocking saved-peer inspection during maintenance, restored state
and unlocked catalog storage at hook entry, and guarded global capture.

Workspace/build, strict Clippy, Rust and Pascal formatting, the standalone Pascal
suite, app/example checks and relative documentation links complete verification.
Reclamation correctness belongs to AP48.6; automatic activation and configurable
initial scheduling belong to AP48.7. Q43 makes controlled performance tuning an
optional follow-up rather than an activation prerequisite.
