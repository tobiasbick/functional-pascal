# Implementation plan: Supervised task retries

> C3 in [concurrency](concurrency.md). Implemented and verified, including C4's compatibility audit.
> The current API contract is in the [Task reference](../../pascal/std/concurrency/task.md).

The supervisor builds on explicit TaskGroup ownership rather than implicitly adopting ordinary
`go` tasks. It does not add language syntax or a hard shutdown guarantee.

## Selected contract

- `StartSupervisedTask(Group, Work, RetryLimit, BackoffMillis)` accepts the same token-parameter
  procedure or function as `StartTaskInGroup` and preserves its task result type.
- The returned task keeps one identity and one group membership across all attempts. Wait,
  WaitAny, and Select observe only its final outcome, never a transient failed attempt.
- RetryLimit counts retries after the initial attempt and is bounded to 0..1023. BackoffMillis
  is a fixed delay between failed attempts, bounded to 0..60000. Zero still permits cooperative
  scheduling and cancellation checks; it does not create recursive retry calls.
- A successful value or procedure return completes the task. A top-level Result Error or Pascal
  panic may retry while budget remains. Other runtime diagnostics are terminal: invalid handles,
  wrong ownership, and invalid operations must not be retried as transient application failures.
- Cancellation takes precedence over starting another attempt. It interrupts pending backoff
  without waiting for the entire configured delay. An executing worker remains cooperative.
- Each new attempt receives the same immutable captures and group token. The supervisor does not
  undo external side effects; restartable workers must account for partial work themselves.
- Successful recovery produces no group failure. Exhaustion preserves the final Result error or
  panic diagnostic in the normal task result and group report. Earlier attempt failures do not
  become separate registered children or accumulate unbounded reports.
- Cancellation before the next attempt produces a terminal Cancelled report. A worker's returned
  Error message is not interpreted as cancellation; the token state controls retry admission.
- Closing the group requests cancellation and joins the supervised child exactly as it joins an
  ordinary group child. Runtime teardown releases retry templates and pending wait registrations.

## Verification evidence

- `crates/fpas-vm/src/vm/debug/tests/supervision.rs` covers normal default-pool, single-worker,
  and deterministic-debugger execution, including nested children and resume-time failures.
- `crates/fpas-vm/src/vm/tasks/supervision/tests.rs` checks admission limits and cancellation
  before executing the worker body.
- `crates/fpas-vm/src/vm/tasks/supervision/lifetime_tests.rs` repeats five direct capture-release
  checks 100 times each, including retained inputs between retries and timer/queue teardown.
- `tests/concurrency/supervised_task_retry_test.fpas` and
  `tests/concurrency/supervised_channel_selection_test.fpas` are bundled FPAS regressions.
- `crates/fpas-vm/src/vm/debug/tests/supervision/channel_compatibility.rs` covers ordinary channel
  operations and nested waits; `entry_completion.rs` verifies forced terminal completion without retry.
- The final complete workspace suite passed with all 535 VM tests. Workspace build, affected-crate
  strict Clippy, standalone FPAS execution, editor generation, and formatting checks passed.

## Coverage requirements

Cover zero retries, success on the first and later attempts, exhaustion, panic versus ordinary
error versus other runtime errors, bounded inputs, cancellation before execution and during
backoff, stable task identity, non-consuming selection, immutable-capture restoration, nested
group ownership, debugger-clock parity, and repeated cleanup. Reuse scheduler and group seams;
do not introduce a thread per supervisor or per retry.
