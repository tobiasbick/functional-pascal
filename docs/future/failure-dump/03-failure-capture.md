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
