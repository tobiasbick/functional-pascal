# Task-group ownership: completed implementation

> C2 is implemented and verified. The current API and behavior are documented in
> [Std.Task](../../pascal/std/concurrency/task.md#task-groups).

Explicit groups own registered children independently of retained task handles. They propagate
cooperative cancellation, collect distinct returned errors and runtime failures, and join children
on creator-only close. Ordinary `go`, task result typing, and nongroup panic handling are unchanged.

## Evidence

- Twenty-two VM tests cover registration and failure-publication races, ownership, bounded groups
  and child counts, report ordering, result consumption, token and membership release, ignored
  report metadata, normal single-worker execution, debugger cancellation, and terminal failures.
- Four semantic tests cover worker signatures, handle types, mutable captures, Result error types,
  and preservation of the routine's result type.
- [Bundled lifecycle regression](../../../tests/concurrency/task_group_lifecycle_test.fpas): nested
  children, discarded handles, explicit cancellation and cancellation during close, final child
  side effects, and idempotent close. Also executed through the deterministic debugger.
- [Worker example](../../../examples/pascal/concurrency/task_group_workers.fpas): cooperative
  worker stop and an ordinary error collected by one group.
- Workspace build and full tests passed, including all 502 VM unit tests and bundled FPAS suites.
  Affected-crate strict Clippy, formatting, editor generation, and the standalone test/example passed.

Group close remains cooperative and has no deadline. Synthetic VM-shutdown results cannot certify
a successful join. The original hard-shutdown requirement remains S1 in the
[concurrency plan](concurrency.md), to be discussed separately after C3 and C4.
