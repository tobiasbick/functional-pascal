# Progress

Last updated: 2026-08-15

## Current checkpoint

Planning is complete. Implementation has not started. The current checkout
already contains exact capture provenance for named routines and intentionally
rejects `Cell` and `EnclosingCell` construction. Begin with `CCRA-01`; do not
skip directly to relaxing that rejection.

## Work-package status

| ID | State | Evidence or next action |
|---|---|---|
| CCRA-01 | planned | Add contract fixture/tests and record current rejection baseline |
| CCRA-02 | planned | Add runtime task-owner invariant before accepting any debugger cell |
| CCRA-03 | planned | Extend exact capture reads without payload access |
| CCRA-04 | planned | Add bounded named-routine construction policy |
| CCRA-05 | planned | Restrict target/frame/task and reuse atomic commit |
| CCRA-06 | planned | Prove sharing, recursion, continuation, and containment |
| CCRA-07 | planned | Add JSONL, DAP, and Extension Host parity |
| CCRA-08 | planned | Update current docs and narrow deferred scope after behavior passes |
| CCRA-09 | planned | Run all gates and close the matrix |

## Planning evidence

Observed on 2026-08-15; this is architecture evidence, not implementation
acceptance evidence:

- `DebugCaptureKind` and compiler metadata already distinguish `Value`, `Cell`,
  and `EnclosingCell` and preserve exact owner binding IDs.
- `Value::Cell` is an `Arc<Mutex<Value>>`; cloning preserves identity.
- `Worker` already exposes a runtime `task_id` internally.
- `SharedFunction` currently stores only a Boolean `task_bound` flag, and call
  entry does not prove an owner task. This gap must be closed before debugger
  injection can be considered contained.
- Named-routine construction currently rejects every mutable capture in
  `mutation/function_value/routine/captures.rs`.
- Generic first-class function copying separately rejects task-bound values and
  must remain unchanged by this package.

## Resume instructions

Run these steps in order after any context loss:

1. Confirm branch and changes:

   ```text
   git branch --show-current
   git status --short
   ```

2. Read:

   ```text
   docs/future/debugger/cell-capturing-routine-assignment/README.md
   docs/future/debugger/cell-capturing-routine-assignment/scope-and-decisions.md
   docs/future/debugger/cell-capturing-routine-assignment/architecture.md
   docs/future/debugger/cell-capturing-routine-assignment/implementation-plan.md
   docs/future/debugger/cell-capturing-routine-assignment/verification-matrix.md
   docs/future/debugger/cell-capturing-routine-assignment/progress.md
   docs/future/debugger/cell-capturing-routine-assignment/consciously-deferred.md
   ```

3. Recheck current file locations and line counts with `rg`; historical paths
   in this plan are routing guidance, not authority.
4. Run focused immutable-capture, function-copy, task-bound closure, artifact,
   JSONL, DAP, and VS Code baselines before editing.
5. Start `CCRA-01`, add failing contract tests, and record exact commands below.
6. After each work package, update its state and only those matrix rows backed
   by current command output.

## Evidence log

No implementation commands are recorded yet. Add dated entries containing the
exact command, exit status, relevant test count or failure, and affected
`CCRA-T*` rows. Keep unrelated baseline failures separate and do not convert an
inspection result into `PASS`.
