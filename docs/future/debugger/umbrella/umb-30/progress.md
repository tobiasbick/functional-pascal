# UMB-30 progress

## Current checkpoint

- Package: `UMB-30` active
- Active work IDs: none; `U30-50` is pending and not started
- Base checkpoint: `b7517403`
- Code changes after base: completed `U30-41` and suppression part of `U30-42`
  remain in the worktree
- Next action: begin `U30-50` only after an explicit continuation request
- Commit/push authorization: none for current worktree changes

## Work status

| ID | Status | Evidence or next action |
|---|---|---|
| `U30-00` | done | Format, locked workspace build, 29 VM, 4 JSONL, and 3 DAP forced-return tests pass at the package base |
| `U30-01` | done | Entry contract frozen; recovery token and initializer-store metadata gaps recorded |
| `U30-10` | done | Root/task entry completion, retained result, task exit, cancellation, and no-dispatch behavior pass |
| `U30-11` | done | JSONL, DAP, VS Code lifecycle, and current docs pass |
| `U30-20` | done | Exact failed-callee and entry recovery transitions preserve the diagnostic and resume correctly |
| `U30-21` | done | JSONL, DAP, real VS Code recovery flow, and current docs pass |
| `U30-30` | done | Typed retained-result replacement passes VM, JSONL, DAP, strict library Clippy, and real VS Code host gates |
| `U30-40` | done | Selected-frame reconstruction retains ABI inputs/captures, resets later state, removes younger frames, and executes nothing at command time |
| `U30-41` | done | Exact local/global initializer identity round-trips through all portable artifacts; one pending store is suppressed only for the exact live frame |
| `U30-42` | done | Existing JSONL/DAP mutation commands and VS Code flows preserve debugger initialization after continue; current docs describe the exact and fallback behavior |
| `U30-50` | pending | Instruction-change feasibility/slice |
| `U30-60` | pending | Full verification and closure |

## Baseline ownership inventory

- `DebugTaskRuntime` owns task slots, dispatch, child-exit events, root result,
  shutdown, and cancellation.
- `TaskScheduler` owns runnable order, retained results/failures, completion
  consumption, waiter relationships, and timers.
- `Worker` owns the live register window, initialization bits, current function
  and instruction pointer, call stack, and suspension state.
- `CallFrame` retains function, return instruction, base, and return
  destination, but no original argument/local snapshot.
- Existing forced return prepares and revalidates a selected live unwind, but
  rejects program/task entry frames and failed/completed task states.
- Scheduler failures are retained and repeatably observable, but have no
  compare-and-transition identity or recovery generation. Successful results
  are consumed once and then represented only by a completion ID.
- Source debug bindings now retain exact verified initializer stores. Local
  stores identify an owner-function `Move` into the binding register; global
  stores identify an owner-function `StoreGlobal` for the exact global slot.
  The identity survives unit objects, linking, and program images.

These are inventory facts, not acceptance of the planned operations.

## Evidence log

```text
2026-08-16 | U30-00 | pending -> active | 1198b1c6 base | completed UMB-20 detail removed; lifecycle contracts, work IDs, layout, and verification rows frozen | run baseline gates
2026-08-16 | U30-00 | active -> done | 1198b1c6 plus docs | format, locked workspace build, 29 VM forced-return tests, 4 JSONL tests, and 3 DAP tests pass | activate U30-01
2026-08-16 | U30-01 | pending -> active | worktree | result/failure ownership and portable source-binding metadata inventoried; exact failure transition and initializer-store identity are absent | add entry-completion atomicity contracts
2026-08-16 | U30-01 | active -> done | worktree | entry ownership, typing, result, exit, cancellation, adapter, and negative contracts frozen | implement U30-10
2026-08-16 | U30-10 | pending -> done | worktree | root/task entry completion publishes retained results once, cancels root peers, emits exits, and dispatches no instruction; 32 VM tests pass | map adapters
2026-08-16 | U30-11 | pending -> done | worktree | 5 JSONL, 4 DAP, real VS Code host, strict Clippy, workspace build, and complete workspace suite pass; debugger docs updated | activate U30-20
2026-08-16 | U30-20 | pending -> active | worktree | scheduler retains failures but lacks exact compare/transition identity | add recovery transition contracts
2026-08-16 | U30-20 | active -> done | 48daa5cd | exact failed callee/entry transitions, atomic type rejection, preserved diagnostics, and resumed output pass | map adapters and editor
2026-08-16 | U30-21 | pending -> done | 48daa5cd | JSONL, DAP, real VS Code host, current docs, strict library Clippy, build, and workspace suite pass | implement retained-result subset
2026-08-16 | U30-30 | pending -> done | worktree | stable retained task result is typed, repeatable before consumption, protocol-equivalent, editor-accessible, and instruction-free; 36 VM, 14 JSONL/DAP, strict library Clippy, and real VS Code host pass | activate U30-40
2026-08-16 | U30-40/42 | active -> done/active | b7517403 | four VM restart tests, paired JSONL/DAP tests, format, strict library Clippy, current docs, and real VS Code host flow pass; restart preserves exact ABI/capture state and dispatches nothing | retain exact initializer-store identity for U30-41
2026-08-16 | U30-41/42 | active -> done | b7517403 plus worktree | exact initializer metadata, verifier rejection, artifact round trips, frame/task-safe one-store suppression, forced-return compatibility, JSONL/DAP parity, VS Code host flow, current docs, format, strict changed-library Clippy, locked workspace build/tests, and diff check pass | wait; U30-50 remains pending
```

## Resume commands

```powershell
git status --short --branch
git diff --stat
Get-Content docs/future/debugger/umbrella/umb-30/progress.md
Get-Content docs/future/debugger/umbrella/umb-30/scope-and-decisions.md
cargo fmt --all -- --check
cargo build --workspace --locked
```

Do not clean, reset, stage, commit, push, merge, or change branches without
matching user authorization.
