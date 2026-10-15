# 03: preserve the original failure

Depends on [02](02-shared-inspection.md). Progress lives in the [overview](README.md).

## Goal and deliverable

Capture the responsible task's state before destructive cleanup and carry its
identity to the final error selected by the existing VM. An optional VM
capture/notification interface lets the CLI receive immutable data before
remaining worker joins can block. The current `Vm::run` error and task ownership
contracts remain intact.

## Implementation boundaries

Inspect [root execution](../../../crates/fpas-vm/src/vm/worker/run.rs),
[pool execution](../../../crates/fpas-vm/src/vm/tasks/pool.rs),
[scope unwind](../../../crates/fpas-vm/src/vm/tasks/scopes/unwind.rs),
[failure selection](../../../crates/fpas-vm/src/vm/tasks/scopes/failures.rs),
[scheduler failures](../../../crates/fpas-vm/src/vm/tasks/scheduler.rs),
[retry handling](../../../crates/fpas-vm/src/vm/tasks/supervision/execution.rs),
[hosted callbacks](../../../crates/fpas-vm/src/vm/hosted/callbacks) and
[VM termination](../../../crates/fpas-vm/src/vm/mod.rs).

Keep capture and failure identity in focused VM modules. Retain the existing
small diagnostic error representation on interpreter hot paths. Propagated
fault identities must survive diagnostic cloning, waits and ordered attachment
of secondary failures. Do not infer relationships by parsing help text.

Captured candidates follow the actual failure's lifetime. Supervisor retries,
handled host background failures and consumed/released outcomes free their
captures. No completed-task history is added. Define how the later task
collector receives a terminal observation request before cleanup blocks.

## Acceptance criteria

- A nested main-task panic retains the actual failing callee and callers,
  parameters and locals from before scope/group cleanup.
- A worker failure propagated through `Wait`, scope exit or group handling
  retains the worker cause, without substituting the waiter or cleanup error.
- Multiple failures select the same primary and secondary ordering as before.
- Hosted callback failures retain their available FPAS continuation chain.
- Handled failures, successful retry, Result data and normal shutdown produce
  no terminal notification; recovered captures do not grow without bound.
- Escaping cancellation and VM diagnostics with no live frame follow the
  explicit unavailable-data rules.
- Capture failure preserves the program diagnostic and normal ownership cleanup.

## Verification to record on delivery

Extend the strongest existing VM failure/ownership tests with observable
retained-cause assertions. Exercise both root and pool paths, retries, callback
failures and competing failures without depending on arbitrary thread order.
Record focused and workspace results, including any unchanged baseline failures.

## Delivered implementation

The opt-in [capture modules](../../../crates/fpas-vm/src/vm/failure_capture)
retain original-fault frames and source bindings through the shared inspection
core, before root or pool scope unwind. `VmError` remains `Box<Diagnostic>`;
a non-rendered, clone-shared diagnostic attachment owns the explicit VM-local
identity. Normal diagnostic rendering and equality are unchanged. Waits and
scope/group cleanup preserve the cause, and secondary relationships are attached
at the existing selection points without parsing help text.

Synchronous hosted callbacks append their actual FPAS callers before those
callers unwind. Scheduled callback result-validation failures inspect the
restored hosted callsite instead of the returned callback's instruction address.
The ordinary runtime diagnostic is preserved in both cases.

Captured data contains detached value trees, not runtime values or mutable
references. Budget leases release retained storage with the owning diagnostics
and observations. Retries and handled group outcomes retain no capture history.
Values and models share the contract's 8 MiB ceiling, with 512 KiB reserved for
control data, including 128 KiB reserved for the final selected diagnostic.
Persistent relationship chains avoid collection copies on diagnostic clones;
relationship nodes, diagnostic copies and terminal model collections are charged
before allocation. At most 1,024 relationships are retained, with explicit omitted
counts and bounded model nesting. Compact identities survive unavailable details.

`Vm::set_failure_observer` delivers one immutable `FailureObservation` only for
an escaping `Vm::run` error. Notification occurs after required ownership barriers
select the error, before remaining server shutdown and worker joins can block.
Errors produced by the final server-completion check follow the same rule with
explicit `no_live_frame` data. The hook is
[`CaptureState::notify`](../../../crates/fpas-vm/src/vm/failure_capture/notification.rs):
step 04 can request the cooperative task observation there before handing the
combined immutable data to the observer. It must not move the selection point
before unfinished owned work can still change the outcome.

The embedded host API is documented in
[shared diagnostics](../../pascal/tools/diagnostics.md#embedded-vm-failure-observation).
FPAS syntax, the authoring reference and skill, task ownership, CLI options and
JSON report publication are unchanged. Task catalogs, globals and peer cuts
remain step 04; CLI/report output remains step 05.

## Verification evidence

- Twelve focused capture tests cover root and pool execution, retained parameters
  and locals, synchronous callback chains, scheduled callback result validation,
  Wait/scope/group propagation, structured group-cleanup secondaries, handled
  group failures, successful supervision after Result and panic attempts,
  diagnostic-clone lifetime, 2,000 competing captures sharing the global budget,
  bounded diagnostic text, escaping shutdown, no-live-frame errors, observer
  panic and notification before a blocked native join. The latter tables root
  failure and server-finalization failure. All twelve pass.
- The existing deterministic scope-failure selection test also verifies retained
  cause IDs and the unchanged primary/secondary order through diagnostic clones.
- Independent subagent review found four issues: uncharged relationship/model
  storage, loss of the primary diagnostic at budget exhaustion, the restored
  scheduled-callback address, and delayed server-finalization notification.
  All four were fixed and re-reviewed; no step-03 blockers remain.
- Three targeted regressions were demonstrated by temporarily removing the
  corresponding fixes: the callback location became unavailable, the selected
  diagnostic was clipped by large values, and server-finalization notification
  timed out behind the native join. Restoring the fixes made all twelve pass.
- `cargo fmt --check`, `cargo build` and the final workspace run pass. The final
  `cargo test --workspace` invocation explicitly filters only the two unchanged
  TCP tests recorded in [step 02](02-shared-inspection.md#verification-evidence):
  `tcp_backpressure_can_time_out_and_cancel_without_closing` and
  `write_deadline_is_not_reset_by_retries_and_preserves_socket_timeouts`.
  All 634 remaining VM unit tests, nine VM integration tests, the CLI/FPAS suites,
  documentation example checks, other workspace packages and doctests pass.
- An earlier unfiltered workspace attempt stopped in the existing timing-sensitive
  JSONL test `instruction_and_timeout_limits_are_reported` (`missing instruction_limit: []`).
  Its owner discards the immediate launch response even though it can contain the
  completed event. The unchanged test passes both an isolated rerun and the final
  workspace run; no debugger protocol or test was changed for this delivery.
- All 79 relative file links in changed Markdown were checked; none are missing.
