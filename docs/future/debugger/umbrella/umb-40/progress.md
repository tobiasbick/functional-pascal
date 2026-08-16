# UMB-40 progress

## Current checkpoint

- Package: `UMB-40` active
- Active work IDs: none; `U40-50` is pending
- Base checkpoint: `c2a264d0`
- Code changes after base: quiescence, all-stop adapters, per-task holds,
  cancel, create/restart rejection, and non-stop/history feasibility rejection
- Next action: begin `U40-50` only after an explicit continuation request
- Commit/push authorization: none for current worktree changes

## Work status

| ID | Status | Evidence or next action |
|---|---|---|
| `U40-00` | done | Format, locked workspace build, 13 VM behavior, 6 JSONL task, 2 DAP task, 10 DAP lifecycle, and 1 scheduler test pass at `c2a264d0` |
| `U40-01` | done | Quiescence contract frozen; inspection cannot dispatch, admit tasks, or move frozen peer instruction windows |
| `U40-10` | done | Catalog is a stop snapshot; resume-only spawn drain; seven VM quiescence tests plus existing pause/priority coverage |
| `U40-11` | done | JSONL `non_stop: false`, paired catalog/stop identity tests, DAP session-wide continue, current docs, VS Code host |
| `U40-20` | done | `pause_task`/`resume_task` holds; schedule skips paused tasks; eight VM task-control tests |
| `U40-21` | done | JSONL `task.pause`/`task.resume`, DAP `fpas/pauseTask`/`fpas/resumeTask`, VS Code commands, current docs |
| `U40-30` | done | `cancel_task` stores `F4016` without dispatch; create/restart reject; five VM lifecycle tests |
| `U40-31` | done | JSONL/DAP/VS Code cancel mapping; create/restart capabilities false; current docs |
| `U40-40` | done | Non-stop, shortcuts, and unbounded history rejected; `task.history` unsupported |
| `U40-50` | pending | Full verification and closure |

## Baseline ownership inventory

- `DebugSession` owns session-wide continue/pause, selected-task stepping,
  cooperative pause, stop generation, and inspection invalidation on resume.
  Stopped-state `pause_task`/`resume_task` set a per-task hold without
  dispatch or inspection invalidation. Stepping a paused task rejects.
  Stopped-state `cancel_task` marks a live non-root task cancelled, stores
  `F4016` for retained waiters, and drops that task's inspection snapshot.
  `create_task` and `restart_task` always reject.
- `DebugTaskRuntime` owns one-lane slots, `schedule`/`dispatch`, spawn drain,
  events, root result, disconnect cancellation, per-task hold flags, and
  stopped-state cancel. `catalog()` is a snapshot and does not call
  `drain_spawned()`; resume `schedule`/`dispatch` remains the only admission
  path. Paused slots are not schedulable, including as wait-dependencies.
  New admitted tasks start unpaused. Cancel does not dispatch or drain
  spawns.
- `TaskScheduler` owns retained results/failures, waiters, timers, and the
  queue that debug resume drains. It does not run the concurrent worker pool
  for a debug session. Debugger cancel uses `store_failure` with `F4016`,
  not `RUNTIME_VM_SHUTDOWN`.
- `Worker` owns registers, IP, frames, and suspension. An already entered
  host intrinsic runs to return before pause is observed.
- JSONL advertises `task_threads: true`, `task_pause: true`,
  `task_cancel: true`, `task_create: false`, `task_restart: false`, and
  `non_stop: false`. Catalog entries include `paused`. DAP
  `supportsSingleThreadExecutionRequests` remains false; continue reports
  `allThreadsContinued: true`. Custom requests `fpas/pauseTask`,
  `fpas/resumeTask`, and `fpas/cancelTask` map a known thread to the JSONL
  commands. VS Code contributes **Debug: Pause Task**, **Debug: Resume Task**,
  and **Debug: Cancel Task**. There is no history command.

## Evidence log

```text
2026-08-16 | UMB-40 | pending -> active | c2a264d0 base | context-loss-safe quiescence package created from current runtime, scheduler, JSONL/DAP all-stop, and host-pause evidence | execute U40-00
2026-08-16 | U40-00 | active -> done | c2a264d0 plus docs | format, locked workspace build, 13 VM behavior, 6 JSONL task, 2 DAP task, 10 DAP, and 1 scheduler test pass | freeze U40-01
2026-08-16 | U40-01 | pending -> done | c2a264d0 plus worktree | all-stop observation frozen; two VM tests prove inspection does not dispatch or admit tasks and frozen peers keep instruction windows | wait; U40-10 remains pending
2026-08-16 | U40-10 | pending -> done | c2a264d0 plus worktree | catalog snapshot without spawn drain; 7 VM quiescence, 13 behavior, 6 JSONL task, 2 DAP task, format, and strict library Clippy pass | wait; U40-11 remains pending
2026-08-16 | U40-11 | pending -> done | c2a264d0 plus worktree | JSONL/DAP all-stop identity tests, `non_stop: false`, current docs, format, Clippy, and VS Code host pass | wait; U40-20 remains pending
2026-08-16 | U40-20 | pending -> done | c2a264d0 plus worktree | VM pause/resume holds; 8 task-control, 7 quiescence, 13 behavior tests; format and strict library Clippy pass | wait; U40-21 remains pending
2026-08-16 | U40-21 | pending -> done | c2a264d0 plus worktree | JSONL/DAP/VS Code per-task holds, `task_pause: true`, current docs, format, Clippy, focused adapter tests, and VS Code host pass | wait; U40-30 remains pending
2026-08-16 | U40-30 | pending -> done | c2a264d0 plus worktree | cancel stores F4016 without command-time dispatch; create/restart reject; 5 VM lifecycle tests | map adapters in U40-31
2026-08-16 | U40-31 | pending -> done | c2a264d0 plus worktree | JSONL/DAP/VS Code cancel, create/restart false, current docs, 4 JSONL + 1 DAP lifecycle tests | record U40-40 rejection
2026-08-16 | U40-40 | pending -> done | c2a264d0 plus worktree | non-stop, shortcuts, and unbounded history rejected; task.history unsupported_capability | wait; U40-50 remains pending
```

## Resume commands

```powershell
git status --short --branch
git diff --stat
Get-Content docs/future/debugger/umbrella/umb-40/progress.md
Get-Content docs/future/debugger/umbrella/umb-40/scope-and-decisions.md
cargo fmt --all -- --check
cargo build --workspace --locked
```

Do not clean, reset, stage, commit, push, merge, or change branches without
matching user authorization.
