# `fpas-vm` review follow-up

Classification: VM runtime, concurrency, and malformed-bytecode defense. No FPAS language change expected. Performance changes require current benchmarks.
Status: VM-01 through VM-05 completed 2026-08-02.

| ID | Priority | Evidence | Finding and impact | Implementation direction | Required regression |
| --- | --- | --- | --- | --- | --- |
| VM-01 | P1 | `crates/fpas-vm/src/vm/execute/mod.rs:251`, `src/vm/worker.rs:217`, `src/vm/execute/concurrency/tasks/wait.rs:41` | `Halt` in a retained spawn task is treated as success but stores no result. The task stays Pending and `Wait` blocks forever. | Permit `Halt` only in the main task with no call frames; otherwise signal an internal bytecode/runtime failure and complete retained task state with error. | Retained spawn reaching Halt must fail diagnostically within a fixed bound, never hang. |
| VM-02 | P1 | `crates/fpas-vm/src/vm/shared.rs:154`, `src/vm/shared/timers.rs:40` | Main-task completion sets global shutdown and clears sleeping tasks even though queued nonsleeping tasks are allowed to continue. Detached sleepers disappear silently. | Separate worker-pool teardown from bytecode-task cancellation and define explicit normal-main-end behavior for timers/tasks. | Detached sleeping task when main halts; assert completion or explicit cancellation according to the documented contract. |
| VM-03 | P2 | `crates/fpas-vm/src/vm/execute/control_calls.rs:179`, `src/vm/execute/mod.rs:225`, `src/vm/mod.rs:79` | Function/image entry equal to `code.len()` is accepted as successful completion, even with active frames; spawn may use an argument as a result. | Validate all entries as strictly less than code length and treat code-end with frames as an invariant error. | Entry at length and above, Call target at length, and code-end with nonempty call stack. |
| VM-04 | P3 | `crates/fpas-vm/src/vm/execute/mod.rs:213`, `src/vm/execute/io/callbacks/sync_call.rs:96`, `src/vm/execute/concurrency/tasks/wait.rs:198` | Main, synchronous callback, and help-run loops duplicate fetch boundaries and Return/Halt/shutdown policy; they already disagree on Halt. | Extract a common step/boundary transition parameterized by explicit execution context. | Existing behavior tests plus shared Halt/Return/boundary cases for every context. |
| VM-05 | P3 | `crates/fpas-vm/src/vm/mod.rs:260` | Lower confidence: `Vm::run` appears single-use because shutdown is not reset, but the public lifecycle contract does not state this. | Model a single-use state explicitly or fully reset reusable runtime state; document the chosen API contract. | Second `run()` on the same VM has a documented deterministic result. |

## Implementation notes

Fix VM-01 and VM-03 after strengthening bytecode/linker validation, but retain VM boundary checks as defense in depth. The existing in-place integer binary hot path is already implemented and regression-covered; this review found no new measured performance fix. If runtime code changes, use targeted VM tests, the workspace suite, and relevant `cargo bench-fpas` comparisons.

## Implementation record

- VM-01 rejects `Halt` outside frame-free main execution. Retained task failures now complete an
  explicit result state containing the original diagnostic, so `Wait` and `WaitAll` propagate the
  actionable task error instead of hanging or replacing it with a generic shutdown message. Batch
  waits inspect completion and failure atomically, including the transition into shutdown.
- VM-02 separates worker-pool teardown from bytecode cancellation. Normal main completion drains
  already-ready tasks and explicitly cancels timer-suspended tasks; attempts to sleep after
  teardown begins are canceled under the same timer lock. Timer dispatch holds that lock until due
  tasks are published to the ready queue, making cancellation a teardown barrier. Retained
  cancellations receive a shutdown diagnostic, while detached sleepers end without a result.
- VM-03 validates image, call, synchronous-callback, and spawn entries with `entry < code.len()`
  before mutating execution state. Reaching the code boundary without `Halt` or `Return` is an
  internal invariant failure in every execution context, including active call frames.
- VM-04 moves fetch-boundary, shutdown, `Halt`, `Return`, and suspension policy into
  `execute/transition.rs`. Main, spawned, synchronous-callback, and helped-task loops consume the
  same context-parameterized transitions.
- VM-05 models `Vm::run` as single-use. A second call returns a deterministic
  `RUNTIME_VM_SHUTDOWN` diagnostic, and the public Rust API documents the contract and errors.
- Main success, main failure, and unwinding use distinct teardown paths. Runtime failures abort
  detached ready work instead of draining it indefinitely.
- `docs/pascal/language/concurrency/` and `docs/pascal/std/concurrency/task.md` now document normal
  teardown cancellation and original task-diagnostic propagation. FPAS syntax is unchanged.

## Verification

- Baseline: `cargo test -p fpas-vm --locked` — passed: 165 tests plus doc tests.
- Baseline: `cargo clippy -p fpas-vm --all-targets --locked -- -D warnings` — passed.
- Baseline: `cargo bench-fpas save before-vm-review --group concurrency` — `task_spawn_wait` 652
  ms, 153374 tasks/s.
- Targeted implementation: `cargo test -p fpas-vm --locked` — passed: 184 tests plus doc tests.
- `cargo clippy -p fpas-vm --all-targets --locked -- -D warnings` — passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — passed.
- `cargo fmt --all -- --check` — passed.
- `cargo build --workspace --locked` — passed.
- `cargo test --workspace --locked --quiet` — passed. The preceding non-quiet attempt reached the
  command timeout after 122 seconds without a test failure; the completed rerun exited successfully.
- Release benchmark comparisons against `before-vm-review` ranged from 620 ms (`-4.9%`) to 669 ms
  (`+2.6%`) across repeated runs. The direction-changing spread is run noise, so no performance
  claim or benchmark-history entry was recorded.
