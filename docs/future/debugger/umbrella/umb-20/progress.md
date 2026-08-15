# UMB-20 progress

## Current checkpoint

- Package state: plan ready; implementation not active
- Active work item: none
- Last completed work item: none
- Next work item: `U20-00`
- Blocker: completed `UMB-10D` changes are still only in the worktree; the
  parent execution rule requires a recoverable checkpoint before activating a
  successor package
- Branch: `codex/fpas-debugger`

Do not create the checkpoint, stage files, clean the worktree, commit, or push
without matching user authorization.

## Work-item status

| ID | Status | Next action |
|---|---|---|
| `U20-00` | pending | Inspect worktree and obtain or use explicit checkpoint authorization |
| `U20-01` | pending | Freeze limits, schemas, and negative contract tests |
| `U20-10` | pending | Split VM modules and implement shared function bindings |
| `U20-11` | pending | Implement JSONL/DAP/VS Code function-breakpoint adapters |
| `U20-20` | pending | Implement shared exact-code runtime-failure policy |
| `U20-21` | pending | Implement JSONL/DAP/VS Code failure-filter adapters |
| `U20-30` | pending | Complete bounded non-mutating policies and ordering |
| `U20-40` | pending | Update current docs and editor surface |
| `U20-50` | pending | Run full verification, checkpoint, and close the package |

Only one work item may be `active`.

## Established baseline inherited from `UMB-10D`

```text
cargo fmt --all -- --check                                  PASS
git diff --check                                            PASS
cargo clippy -p fpas-vm -p fpas-debug --lib -- -D warnings PASS
cargo build --workspace --locked                            PASS
focused VM function_value_assignment                        PASS (23)
focused JSONL function_value_assignment                     PASS (3)
focused DAP function_value_assignment                       PASS (4)
VS Code extension test suite                                PASS
cargo test --workspace --locked --no-fail-fast              BASELINE
  only repository_context reference-count assertion fails: actual 23, expected 22
```

Refresh this evidence in `U20-00`; do not assume it remains current after
additional worktree changes.

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
```

Append one line for every state change. Do not record usernames, hostnames,
absolute user paths, process IDs, or other machine-identifying data.
