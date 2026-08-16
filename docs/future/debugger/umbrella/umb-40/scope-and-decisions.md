# UMB-40 scope and decisions

## Shared invariants

1. The debug session remains launch-owned and all-stop until `UMB-40D`
   records an evidence-backed non-stop decision. A stop freezes every live
   task at a complete bytecode instruction boundary.
2. Runtime task IDs are identities. DAP thread IDs are a stable adapter map
   of those identities. Display names are not identities.
3. Inspection, mutation, evaluation, and lifecycle commands observe or mutate
   only the current stop generation. A rejection changes no worker, scheduler
   result, waiter, stop generation, or adapter state.
4. JSONL and DAP call the same session operation. VS Code maps to that adapter
   behavior and must not reconstruct VM state itself.
5. No FPAS syntax, semantics, or language documentation changes are in scope.
6. Unsafe thread termination is forbidden as pause or cancellation.

## Current ownership inventory

These are inventory facts for `U40-00`, not acceptance of later children.

- `DebugSession` owns session state (`Stopped` / `Running` / `Failed` /
  `Terminated`), the cooperative pause flag, stop generation, and resume
  policy. Continue and pause are session-wide. Step in/over/out select one
  task and may dispatch wait-dependencies until that task becomes runnable or
  another task hits a higher-priority stop. Stopped-state `pause_task` and
  `resume_task` set a per-task hold without dispatch; a paused task is not
  scheduled, including as a wait-dependency. Stopped-state `cancel_task`
  cancels one live non-root task without dispatch.
- `DebugTaskRuntime` owns task slots, single-instruction dispatch, spawn
  drain, child-exit events, root result, shutdown, and disconnect
  cancellation. Debug execution uses one host lane; it does not use the
  concurrent worker pool. Stopped `catalog()` is a snapshot and does not
  drain queued spawns or refresh waiter readiness.
- `TaskScheduler` owns runnable enqueue/dequeue, retained results/failures,
  completion consumption, waiter relationships, and timers. The debug driver
  polls it; it does not independently dispatch bytecode during a stop.
- `Worker` owns the live register window, instruction pointer, call stack,
  and suspension state. Hosted blocking work that has already entered an
  intrinsic runs to return; pause is observed at the next instruction or
  source boundary afterward.
- JSONL `tasks` pages the stopped catalog and reports `paused`. JSONL
  continue/pause have no `task_id` and ignore extra task arguments. JSONL
  steps accept optional `task_id`. JSONL `task.pause`/`task.resume` require a
  current `task_id`. JSONL `task.cancel` requires a current non-root
  `task_id`. JSONL advertises `non_stop: false`, `task_pause: true`,
  `task_cancel: true`, and `task_create`/`task_restart` false. DAP
  `threads` maps main task `0` to thread `1` and later tasks to stable
  positive IDs. `supportsSingleThreadExecutionRequests` is false; continue
  reports `allThreadsContinued: true`; stopped events report
  `allThreadsStopped: true`. `fpas/pauseTask`/`fpas/resumeTask` map a known
  thread onto the JSONL holds. `fpas/cancelTask` maps a known thread onto
  JSONL cancel. A paused DAP thread name includes `[paused]`.

## `UMB-40A` — quiescence protocol

- **All-stop ownership.** While the session is stopped, failed, or
  terminated, no task may execute `dispatch_one`. Catalog, stack, scopes,
  variables, evaluate, and mutation remain available only in inspectable
  stopped or failed states. Resume invalidates inspection once before any
  further dispatch.
- **Stop owner.** The task that caused the stop is recorded as `task_id`.
  Peer tasks remain frozen at their last completed instruction. They are
  inspectable when their lifecycle state retains a worker snapshot; completed
  and cancelled tasks stay catalog-visible briefly and have no fabricated
  stack.
- **Shared-state observation.** Globals, cells, and other shared values are
  observed only after every live task is frozen at an instruction boundary.
  A supposedly stopped peer must not mutate observed state. Waiter readiness
  may be refreshed only as part of an explicit resume, never as a hidden
  side effect of inspection.
- **Scheduler handoff.** During a stop, `TaskScheduler` may retain results,
  failures, waiters, and timers, but it must not run workers. Debug resume
  remains the only path that drains the scheduler queue into
  `DebugTaskRuntime` slots and dispatches instructions. Round-robin among
  runnable tasks uses stable task-ID order after `last_dispatched`.
- **Blocked host work.** A pause or execution-limit request that arrives
  while a host intrinsic is already in progress is cooperative. It is
  observed after that intrinsic returns, at the next source or instruction
  boundary. `UMB-50D` owns interruption inside blocking host calls; this
  child must not invent unsafe cancellation.
- **Priority while running.** A breakpoint, runtime failure, pause, or
  resource limit from any progressed task wins over the selected step target.
  That existing resume policy is part of all-stop, not per-task pause.
- **Proof before mutation.** `UMB-40A` locks the current all-stop contract
  with tests. It does not add per-task pause, create/cancel/restart commands,
  non-stop mode, or retained execution history. `UMB-50` and `UMB-70` may
  start only after this proof is recorded.

## `UMB-40B` — per-task pause and resume

- Begins only after `UMB-40A` tests prove that stopped peers cannot execute.
- A per-task pause or resume must name a current runtime task ID. Unknown,
  stale, completed, and cancelled identities reject atomically.
- Pause and resume are stopped-state holds. They do not dispatch, drain
  spawns, or invalidate the current inspection generation. Session-wide
  continue/pause remain the default resume commands.
- A paused task is never dispatched, including as a wait-dependency of a
  stepping peer. Newly admitted tasks start unpaused. Stepping a paused
  task rejects without resume. If continue or a peer step cannot make
  progress without running a paused task, the session stops immediately
  with pause and no further dispatch.
- DAP must not advertise single-thread execution until the VM contract and
  JSONL command exist.
- JSONL and DAP must report equivalent task identity, stop owner, and
  all-stop or selected-task state.

## `UMB-40C` — task creation, cancellation, and restart

- Begins only after `UMB-40A`. Prefer extending existing lifecycle and
  scheduler result identities from `UMB-30` instead of a second task-control
  engine.
- Creation, cancellation, and restart must define result handles, waiters,
  cleanup, propagation, and deterministic errors before any successful path.
- A successful command executes no bytecode, host I/O, or waiter body.
  Newly ready tasks become runnable only after an explicit continue or
  accepted resume policy.
- Root cancellation remains the existing disconnect/root-completion path
  unless a narrower debugger command is proven necessary.
- **Accepted subset.** `task.cancel` names a current non-root live task,
  marks it `cancelled`, emits `Exited`, and stores `F4016`
  (`RUNTIME_TASK_CANCELLED`) when the task retains a result. Unknown,
  completed, cancelled, failed, and root identities reject atomically. The
  command does not dispatch, drain spawns, or refresh waiters. Waiters observe
  the stored failure on the next continue.
- **Rejected subset.** Debugger task creation and task restart have no
  successful path. Commands exist so clients receive `task_create_unsupported`
  and `task_restart_unsupported`. Capabilities `task_create` and
  `task_restart` are false. Frame restart remains the proven reconstruction
  path; programs spawn work with `go`.

## `UMB-40D` — non-stop, scheduler shortcuts, and retained history

- Separate bounded feasibility gates after all-stop controls are stable.
- **Rejected.** Non-stop execution cannot prove shared-state observation
  without freezing every live task. A best-effort dirty read is forbidden.
  JSONL `non_stop` remains false.
- **Rejected.** Cross-task stepping shortcuts would hide wait-dependency
  dispatch or violate stop priority and stable task-ID order. Resume keeps
  the existing one-lane schedule.
- **Rejected.** Unbounded retained history is out of scope. Completed and
  cancelled tasks remain briefly visible for lifecycle reporting only.
  Persistent recordings belong to `UMB-80`. There is no `task.history`
  command.

## Bounds

- Existing debugger instruction, timeout, output, inspection, and evaluation
  limits remain authoritative and cover combined work of the entire session.
- Completed and cancelled tasks remain briefly visible for lifecycle
  reporting; this package must not grow an unbounded historical catalog by
  default.
- Any new protocol field has a stable JSONL spelling, a DAP mapping, and a
  focused negative test.

## Out of scope

- Hosted terminal/TUI/graph transport and pause-in-host (`UMB-50`).
- Data breakpoints and capture-cell identity (`UMB-70`).
- Attach/remote (`UMB-60`), record/replay (`UMB-80`), hot reload (`UMB-90`).
- Arbitrary instruction-pointer changes (rejected by `UMB-30D`).
