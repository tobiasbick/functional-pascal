# `fpas-vm` review follow-up

Classification: VM runtime, concurrency, and malformed-bytecode defense. No FPAS language change expected. Performance changes require current benchmarks.
Status: all findings open.

| ID | Priority | Evidence | Finding and impact | Implementation direction | Required regression |
| --- | --- | --- | --- | --- | --- |
| VM-01 | P1 | `crates/fpas-vm/src/vm/execute/mod.rs:251`, `src/vm/worker.rs:217`, `src/vm/execute/concurrency/tasks/wait.rs:41` | `Halt` in a retained spawn task is treated as success but stores no result. The task stays Pending and `Wait` blocks forever. | Permit `Halt` only in the main task with no call frames; otherwise signal an internal bytecode/runtime failure and complete retained task state with error. | Retained spawn reaching Halt must fail diagnostically within a fixed bound, never hang. |
| VM-02 | P1 | `crates/fpas-vm/src/vm/shared.rs:154`, `src/vm/shared/timers.rs:40` | Main-task completion sets global shutdown and clears sleeping tasks even though queued nonsleeping tasks are allowed to continue. Detached sleepers disappear silently. | Separate worker-pool teardown from bytecode-task cancellation and define explicit normal-main-end behavior for timers/tasks. | Detached sleeping task when main halts; assert completion or explicit cancellation according to the documented contract. |
| VM-03 | P2 | `crates/fpas-vm/src/vm/execute/control_calls.rs:179`, `src/vm/execute/mod.rs:225`, `src/vm/mod.rs:79` | Function/image entry equal to `code.len()` is accepted as successful completion, even with active frames; spawn may use an argument as a result. | Validate all entries as strictly less than code length and treat code-end with frames as an invariant error. | Entry at length and above, Call target at length, and code-end with nonempty call stack. |
| VM-04 | P3 | `crates/fpas-vm/src/vm/execute/mod.rs:213`, `src/vm/execute/io/callbacks/sync_call.rs:96`, `src/vm/execute/concurrency/tasks/wait.rs:198` | Main, synchronous callback, and help-run loops duplicate fetch boundaries and Return/Halt/shutdown policy; they already disagree on Halt. | Extract a common step/boundary transition parameterized by explicit execution context. | Existing behavior tests plus shared Halt/Return/boundary cases for every context. |
| VM-05 | P3 | `crates/fpas-vm/src/vm/mod.rs:260` | Lower confidence: `Vm::run` appears single-use because shutdown is not reset, but the public lifecycle contract does not state this. | Model a single-use state explicitly or fully reset reusable runtime state; document the chosen API contract. | Second `run()` on the same VM has a documented deterministic result. |

## Implementation notes

Fix VM-01 and VM-03 after strengthening bytecode/linker validation, but retain VM boundary checks as defense in depth. The existing in-place integer binary hot path is already implemented and regression-covered; this review found no new measured performance fix. If runtime code changes, use targeted VM tests, the workspace suite, and relevant `cargo bench-fpas` comparisons.
