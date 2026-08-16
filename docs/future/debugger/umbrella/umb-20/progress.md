# UMB-20 progress

## Current checkpoint

- Package state: implementation verified; checkpoint pending
- Active work item: `U20-50`
- Last completed work item: `U20-40`
- Next work item: authorized recoverable checkpoint, then delete this detail package
- Blocker: Git checkpoint requires explicit user authorization
- Branch: `codex/fpas-debugger`

Do not create the checkpoint, stage files, clean the worktree, commit, or push
without matching user authorization.

## Work-item status

| ID | Status | Next action |
|---|---|---|
| `U20-00` | done | Recoverable remote checkpoint and green baseline established |
| `U20-01` | done | Limits, schemas, catalog validation, and negative contracts are frozen |
| `U20-10` | done | Shared exact-identity function binding and ordered stops implemented |
| `U20-11` | done | JSONL, standard DAP, and VS Code function breakpoints implemented |
| `U20-20` | done | Shared exact-code runtime-failure policy and failed termination implemented |
| `U20-21` | done | JSONL, standard DAP, and VS Code failure-filter adapters implemented |
| `U20-30` | done | Conditions, hit counts, log ordering, bounds, and action rejection verified |
| `U20-40` | done | Current debugger, JSONL, and DAP docs updated |
| `U20-50` | active | Verification passes; create authorized checkpoint and remove this package |

Only one work item may be `active`.

## Refreshed `U20-00` baseline

```text
cargo fmt --all -- --check                                  PASS
git diff --check                                            PASS
cargo clippy -p fpas-vm -p fpas-debug --lib -- -D warnings PASS
cargo build --workspace --locked                            PASS
focused VM function_value_assignment                        PASS (23)
focused JSONL function_value_assignment                     PASS (3)
focused DAP function_value_assignment                       PASS (4)
VS Code extension test suite                                PASS
cargo test --workspace --locked --no-fail-fast              PASS
```

The former reference-count baseline was a stale test oracle: a Notes fixture
had gained one real `NotesUpdate` reference. The corrected 23-total/22-fixture
expectations pass both focused and workspace tests. Do not assume this baseline
remains current after additional worktree changes.

## Implemented layout

The planned ownership split was followed with two smaller deviations:

- VM function identity, recursion, task, ordering, and limit cases fit in the
  focused `tests/breakpoints.rs`; a second nested test directory was not needed.
- JSONL function breakpoint handling fits in the focused sibling module
  `server/function_breakpoints.rs`; no otherwise-empty `server/breakpoints/`
  directory was introduced.

`session.rs` and `dap/server.rs` are each 441 lines after their breakpoint
responsibilities were split into focused modules.

## Resume commands

Run from the repository root:

```powershell
git status --short --branch
git diff --stat
Get-Content docs/future/debugger/umbrella/umb-20/README.md
Get-Content docs/future/debugger/umbrella/umb-20/progress.md
Get-Content docs/future/debugger/umbrella/umb-20/implementation-plan.md
cargo fmt --all -- --check
git diff --check
```

## Evidence log

```text
2026-08-15 | package created | worktree | UMB-10D reviewed and its obsolete detail plan removed; UMB-20 contracts, work IDs, layout, matrix, and checkpoint gate recorded | perform U20-00 after checkpoint authorization
2026-08-16 | U20-00 pending -> done | 87fd3b98 plus worktree | clean remote checkpoint; format, diff, Clippy, build, 23 VM, 3 JSONL, 4 DAP, VS Code, corrected reference test, and full workspace suite pass | activate U20-01
2026-08-16 | U20-01 pending -> active | worktree | authoritative VM, JSONL, DAP, diagnostics, policy, and file-size inventory captured | add negative contract tests
2026-08-16 | U20-01 active -> done | worktree | selector, filter, schema, resource-bound, atomicity, and unsupported-action contracts encoded | implement shared function engine
2026-08-16 | U20-10 pending -> done | worktree | exact FunctionId binding, multi-match ordering, recursion, task hits, source collisions, and logical/physical limits pass | expose adapters
2026-08-16 | U20-11 pending -> done | worktree | JSONL replace-all, standard DAP setFunctionBreakpoints, and VS Code FunctionBreakpoint pass | implement failure policy
2026-08-16 | U20-20 pending -> done | worktree | central runtime diagnostic catalog, default/exact/empty selection, and nonmatching failed termination pass | expose filters
2026-08-16 | U20-21 pending -> done | worktree | JSONL filters, DAP exceptionBreakpointFilters, and VS Code selection/exit sequence pass | complete policies
2026-08-16 | U20-30 pending -> done | worktree | function conditions/hit counts, same-boundary log-before-stop, bounds, and mutating-action rejection pass | update current docs
2026-08-16 | U20-40 pending -> done | worktree | debugger, JSONL, and DAP pages describe only implemented selectors, filters, limits, and events | verify package
2026-08-16 | U20-50 pending -> active | worktree | format, diff, strict changed-library Clippy, workspace build/tests, focused tests, and real VS Code host pass | obtain checkpoint authorization, then remove detail package
```

Append one line for every state change. Do not record usernames, hostnames,
absolute user paths, process IDs, or other machine-identifying data.
