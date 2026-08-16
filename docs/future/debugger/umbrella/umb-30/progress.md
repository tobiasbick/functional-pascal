# UMB-30 progress

## Current checkpoint

- Package: `UMB-30` active
- Active work ID: `U30-20`
- Base checkpoint: `1198b1c6`
- Code changes after base: U30-10/11 root/task entry completion is implemented
  and fully verified in the worktree
- Next action: add exact scheduler failure compare/transition primitives and
  recovery atomicity tests before enabling failed-frame replacement
- Commit/push authorization: none for current worktree changes

## Work status

| ID | Status | Evidence or next action |
|---|---|---|
| `U30-00` | done | Format, locked workspace build, 29 VM, 4 JSONL, and 3 DAP forced-return tests pass at the package base |
| `U30-01` | done | Entry contract frozen; recovery token and initializer-store metadata gaps recorded |
| `U30-10` | done | Root/task entry completion, retained result, task exit, cancellation, and no-dispatch behavior pass |
| `U30-11` | done | JSONL, DAP, VS Code lifecycle, and current docs pass |
| `U30-20` | active | Add exact retained-failure transition and recovery atomicity tests |
| `U30-21` | pending | Recovery adapters/editor/docs |
| `U30-30` | pending | Completed-result replacement decision/slice |
| `U30-40` | pending | Frame restart shared engine |
| `U30-41` | pending | Source-initializer suppression proof/slice |
| `U30-42` | pending | Restart/suppression adapters/editor/docs |
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
- Existing source debug bindings retain declaration identity, register, scope,
  type, mutability, visibility, and capture provenance. They do not retain the
  exact instruction that performs a source initializer store, so initializer
  suppression cannot be implemented by current metadata alone.

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
