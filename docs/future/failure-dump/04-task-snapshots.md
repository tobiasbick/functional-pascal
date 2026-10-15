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

## Delivered implementation

The optional [task catalog](../../../crates/fpas-vm/src/vm/failure_capture/catalog)
tracks stable identities across queue, worker, suspension, timer, result retention
and release. Saved state is moved into synchronized ownership, never duplicated.
Inspection temporarily borrows that original state and restores it by RAII;
it executes no queued instructions and consumes no channel values or task results.
Released outcomes are removed instead of retained as history. Terminal pool
disposal distinguishes a cancelled worker from a native operation still running.
A converted host panic marks its departed worker failed without inventing frames.
Expired plain sleeps become runnable when the real timer publishes them to the
queue; poll waits and pending supervision backoffs retain their waiting state.

Running peers answer requests at ordinary bounded dispatch boundaries. All peers
share one 500 ms deadline; an unresponsive native peer retains its identity and
known state with explicit unavailable data. Peers continue immediately after
copying their bounded observation; collection never pauses them. Observer delivery
occurs after collection guards and runtime locks have been released. Original
faults, the main task's pre-cleanup propagation stack and later peer observations
remain distinct. Synchronous callback peer data explicitly reports unavailable
outer storage when only its current callback worker can answer.

The catalog keeps at most 1,024 entries, preferring the primary cause, main task,
other known causes and then ascending peer IDs. Output is sorted by task ID.
Known structured causes remain included in task totals even after their scheduler
owners release them or selection omits their entries. Omission counts and reasons
are explicit. Completed/cancelled peers receive no fabricated stack. Handled
supervision attempts release their propagation data before retry/completion.

Globals are read once from the actual root worker's local or shared slots. The
root retains that storage through server finalization and worker joins, including
errors first selected when joining a panicked worker. Such a later peer failure
does not change a successfully completed main task into a failed main task.

The shared 8 MiB data ceiling now reserves 2 MiB for control/model storage,
including the existing 128 KiB terminal allowance, so large values cannot consume
the catalog's metadata allowance. Capture-disabled tasks retain direct scheduler
storage and a dispatch specialization without D1 polling. Successful enabled runs
release all catalog charges. The embedded API is documented in
[shared diagnostics](../../pascal/tools/diagnostics.md#embedded-vm-failure-observation).
FPAS syntax, the compact reference, authoring skill and CLI are unchanged.

## Verification evidence

- Controlled scheduler tests cover runnable, running, waiting, sleeping, failed,
  retained completed/cancelled and already released tasks. Queued inspection proves
  that pending code does not execute and buffered channel/task values remain intact.
- Sixteen concurrent repetitions exercise ordinary spawn, wait, completion, release
  and worker handoffs. Large-set tests cover 1,200 queued tasks with a cause outside
  the admitted prefix and 1,024 structured secondaries whose omitted identities
  still contribute to catalog totals.
- An isolated child process blocks a real host operation, observes its unavailable
  peer entry and signals report delivery before the parent releases that operation.
  Independent timeouts and cleanup bound both processes. The child fixture is
  marked ignored for direct suite discovery and explicitly executed by its parent.
- Independent subagent review identified stale running state after abort, propagation
  snapshots retained across supervision retries, a lost main-task secondary fault,
  and missing globals after a converted pool panic. All were repaired. Five focused
  regressions, including the cause-total bound, failed for the intended reasons
  before their corresponding repairs. The existing Wait/scope ownership test also
  exposed and now covers cancellation before child-cause selection.
- The 25 focused capture tests pass. The single directly ignored child fixture
  is executed by its passing process-isolation parent. Both sensitive scope/cause
  regressions also pass 16 independent process repetitions each (32 runs).
- A sixth pre-fix regression proves the expired-sleep handoff through the real
  timer and scheduler notification, without dequeueing or dispatching the task.
  It failed with stale `sleeping` state and passes with `runnable` after repair.
  The final full VM suite passes: 647 unit tests and nine integration tests,
  with the same two documented baseline TCP tests excluded. Independent review
  also checked the final timer handoff and its read-only supervision accessor.
- The success smoke check executes 32 fresh VMs per mode from the same compiled
  image, each checking a 20,000-iteration arithmetic loop and released storage.
  An isolated unoptimized test-profile sample measured 197.31 ms disabled and
  181.25 ms enabled (ratio 0.919). This noisy development sample establishes the
  measurement path, not a speedup or zero-overhead claim. The release-mode,
  workload-by-workload performance comparison remains owned by step 06.
- Formatting and workspace build checks pass. The final workspace run passes
  the 586 CLI tests, including the FPAS suites and documentation examples,
  debugger protocols, other packages and doctests. It filters only the two
  unchanged TCP tests documented in
  [step 02](02-shared-inspection.md#verification-evidence). After the last timer
  regression/repair, the entire VM suite was rerun with the 647/9 passing counts
  above; the workspace build was repeated for that final code. No language or
  CLI behavior outside the opt-in embedded observer changed.
- All 80 relative file links in the changed Markdown resolve. The compact
  reference, authoring skill, AGENTS instructions and root README need no
  changes for this embedded Rust API delivery; CLI documentation follows in 05.
