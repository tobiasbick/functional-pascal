# UMB-40 implementation plan

## Intended code layout

The session root, execution resume loop, task driver, scheduler, and DAP
server root are already near or above 300–430 lines. New behavior must enter
focused modules instead of extending those mixed roots.

```text
crates/fpas-vm/src/vm/debug/
  tasks/
    driver.rs              — exists: schedule, dispatch, resume-only drain
    driver/
      completion.rs        — exists: entry completion
      recovery.rs          — exists: failure recovery
      completed_result.rs  — exists: retained-result replacement
      quiescence.rs        — exists: stopped catalog snapshot; no inspection drain
      control.rs           — exists: per-task pause/resume hold flags
      lifecycle.rs         — exists: stopped-state cancel of one non-root task
  session/
    execution.rs           — exists: continue/step/pause loop
    tasks.rs               — exists: DebugSession pause/resume/cancel facade
    lifecycle.rs           — exists: create/restart rejection
crates/fpas-debug/src/jsonl/server/
  tasks.rs                 — exists: stopped catalog paging
  task_control.rs          — exists: pause/resume/cancel mapping
  lifecycle.rs             — exists: create/restart rejection
crates/fpas-debug/src/dap/server/
  tasks.rs                 — exists: runtime-task to DAP-thread map
  task_control.rs          — exists: fpas/pauseTask, resumeTask, cancelTask
  lifecycle.rs             — exists: fpas/createTask and fpas/restartTask rejection
editors/vscode/src/debugger/
  taskControlCommand.ts    — exists: Debug: Pause Task / Resume Task / Cancel Task
```

Do not add history modules; `UMB-40D` rejected unbounded retained history.

## Ordered work

| ID | Status | Work | Exit gate |
|---|---|---|---|
| `U40-00` | done | Verify checkpoint `c2a264d0`, focused task baseline, file sizes, and current runtime/scheduler/adapter ownership | Recorded clean-code baseline; documentation-only transition is explicit |
| `U40-01` | done | Freeze quiescence contracts; inventory shared-state observation, blocked host work, and scheduler handoff; add rejection tests for hidden peer execution | All-stop observation is frozen; later children remain explicitly out of this slice |
| `U40-10` | done | Prove `UMB-40A`: no dispatch while stopped, frozen peers, stop-generation inspection, cooperative host pause, scheduler poll-only during stop | VM tests lock the quiescence protocol; gaps are fixed in the shared engine |
| `U40-11` | done | Map proven all-stop fields through JSONL, DAP, and current docs if protocol wording is incomplete | Adapter parity for `all_tasks_stopped` / `allThreadsStopped` and catalog identity |
| `U40-20` | done | Implement `UMB-40B` per-task pause/resume only after `U40-10` | Paused peers cannot execute; unknown/stale IDs reject atomically |
| `U40-21` | done | Map per-task control through JSONL, then DAP/VS Code | Identity parity; no single-thread DAP capability until the VM contract exists |
| `U40-30` | done | Implement the provable `UMB-40C` create/cancel/restart subset | Result handles, waiters, cleanup, and deterministic errors; no hidden execution |
| `U40-31` | done | Map accepted lifecycle-control commands through adapters/editor | Protocol-equivalent success and negatives |
| `U40-40` | done | Run `UMB-40D` feasibility for non-stop, shortcuts, and retained history | Positive safe subset or explicit rejection/dependency is recorded |
| `U40-50` | pending | Run full verification, reconcile docs, and checkpoint/package closure | All applicable matrix rows pass and parent evidence is complete |

## Test placement

- VM quiescence and task-control tests belong in focused files named for
  all-stop, per-task pause, cancellation, and history decisions.
- JSONL and DAP tests must pair the same scenario and assert equivalent
  stable error codes, events, and resulting state.
- VS Code extension-host tests exercise threads, all-stop, and editor
  commands; they do not duplicate VM invariants.
- Scheduler tests cover waiter/result transitions that debug resume must not
  execute at command time.

## Per-work-item procedure

1. Recheck branch, worktree, active ID, and named prerequisites.
2. Inspect target directory shape and line counts; record any layout change.
3. Add negative and atomicity tests first.
4. Implement the smallest shared-engine slice, then adapters.
5. Run focused format/build/tests and update [progress.md](progress.md).
6. Do not stage, commit, push, merge, or activate the next primary package
   without matching authorization and a recoverable checkpoint.
