# 04: capture every relevant task

Depends on [03](03-failure-capture.md). Progress lives in the [overview](README.md).

## Goal and deliverable

Provide the full task catalog and bounded peer frame/value captures required
by D1, using the ordinary concurrent worker pool. A failing-task-only report
does not satisfy this step.

## Implementation boundaries

Inspect [task state](../../../crates/fpas-vm/src/vm/tasks/state.rs),
[worker batches](../../../crates/fpas-vm/src/vm/worker/run.rs),
[scheduler](../../../crates/fpas-vm/src/vm/tasks/scheduler.rs),
[pool handoff](../../../crates/fpas-vm/src/vm/tasks/pool/suspension.rs),
[suspension](../../../crates/fpas-vm/src/vm/tasks/suspension.rs),
[task groups](../../../crates/fpas-vm/src/vm/tasks/groups),
[task scopes](../../../crates/fpas-vm/src/vm/tasks/scopes) and
[hosted runtime](../../../crates/fpas-vm/src/vm/hosted).

Add an optional capture coordinator and lifecycle catalog with explicit
ownership through queue/worker/waiter/timer transitions. A task appears exactly
once at the observation cut. Current retained terminal outcomes may be listed;
already released task history is not reconstructed.

Freeze/read running peers only at safe existing dispatch boundaries. Read
suspended task state under its owner's synchronization. Collect owned bounded
snapshots, then release coordination before invoking a report writer. Never
hold scheduler, value or global locks across cross-task waits or file I/O.
Every exit path releases paused peers and allows normal cleanup to proceed.

## Acceptance criteria

- Root, cause, runnable, running, sleeping, waiting, failed and retained
  completed/cancelled tasks receive the specified catalog entries.
- A blocked native operation yields an unavailable peer capture within the
  single collection deadline, while its identity and known state remain visible.
- Tasks racing spawn, suspension, completion, cleanup or worker handoff are
  neither duplicated nor silently lost within the captured set.
- Task and report limits preserve primary/root identities and disclose omissions.
- Cause-time and later peer observations remain distinguishable; data is not
  presented as a simultaneous global snapshot.
- Collection executes no queued FPAS work, consumes no task result or channel
  value, creates no deadlock and never enables the debugger scheduler.
- Successful executions without the flag allocate no D1 task catalog and keep
  the normal dispatch path free of D1-specific polling.

## Verification to record on delivery

Use controlled task/channel barriers for lifecycle races and observable peer
states. Test a blocking host operation in an isolated process with independent
test cleanup, proving that report availability does not wait for that operation
to finish. Bound the test harness too. Add stress coverage for repeated
failures and large task sets; record results and measured overhead.
