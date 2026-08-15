# Umbrella progress

## Current checkpoint

- Umbrella state: implementation active
- Active primary package: `UMB-10`
- Last completed item: `UMB-10A`
- Next child: `UMB-10B`
- Branch: `codex/fpas-debugger`
- Implementation started: yes

The working tree contains the umbrella plan and the completed `UMB-10A`
implementation. Nothing in this umbrella checkpoint has been committed.
Inspect the current worktree before changing or staging anything.

## Package status

| ID | Status | Last evidence or next action |
|---|---|---|
| `UMB-00` | done | Current branch/worktree and focused baseline verified; CCRA is recoverable at `bed152a2` |
| `UMB-01` | done | Child contracts, dependencies, risks, and acceptance evidence frozen in this umbrella |
| `UMB-10` | active | `UMB-10A` done; prepare tests and provenance contract for `UMB-10B` |
| `UMB-20` | pending | Contract tests for function IDs and diagnostic filters |
| `UMB-30` | pending | Scheduler/waiter design review before code |
| `UMB-40` | pending | Quiescence protocol first |
| `UMB-50` | pending | Transport separation design after `UMB-40A` |
| `UMB-60` | pending | Local attach before remote; native is go/no-go |
| `UMB-70` | pending | Stable data identities before watchpoints |
| `UMB-80` | pending | Recording format and nondeterminism inventory first |
| `UMB-90` | pending | Requires version/snapshot model from `UMB-80` |
| `UMB-99` | pending | Final parity and plan removal |

Allowed statuses are `pending`, `active`, `blocked`, `rejected`, and `done`.
Only one primary package may be `active`.

## Known baseline evidence

The immediately preceding debugger review established:

- Rust formatting, diff checks, workspace build, focused bytecode/VM/debugger
  tests, FPAS fixture formatting, and VS Code extension-host tests passed.
- The full workspace test completed with one independently known
  language-service failure:
  `repository_references_find_notes_update_in_the_consuming_program` reports
  23 references while its assertion expects 22.
- Strict library Clippy for the changed VM and debugger libraries passed.
  Workspace/all-target Clippy is not a gate for this slice because existing
  test/example findings are outside its changed library scope.

The current `UMB-10A` work remains an uncommitted worktree checkpoint. Do not
stage, commit, merge, or push it without matching user authorization.

## Child evidence

### `UMB-10A` — done in worktree

- Exact `SharedFunction` identity and existing cell handles are preserved.
- Copy is limited to a mutable local or parameter register in the selected
  owner task and selected frame.
- Foreign owner, global, descendant, capture-cell, stale-frame, spawn, and
  foreign invocation cases are rejected without a partial write.
- JSONL and DAP use the shared VM/session implementation; the VS Code extension
  uses standard Variables/Watch mutation and emits negotiated invalidation.
- No compiler, bytecode metadata, FPAS syntax, semantics, or language page was
  changed.

Current evidence:

```text
cargo fmt --check                                           PASS
fpas fmt --check (changed debugger fixture)                 PASS
cargo build --workspace --locked                            PASS
cargo clippy -p fpas-vm -p fpas-debug --lib -- -D warnings PASS
focused VM function/cell-capture tests                      PASS (18 + 17)
focused JSONL/DAP cell-capture tests                        PASS (2 + 2)
VS Code extension-host suite                                PASS
cargo test --workspace --locked --no-fail-fast              BASELINE
  only repository_context reference-count assertion fails: actual 23, expected 22
git diff --check                                            PASS
```

Evidence log:

```text
2026-08-15 | UMB-00 | pending -> done | worktree | branch, scope, format, focused tests, build, and independent workspace baseline verified | freeze umbrella contracts
2026-08-15 | UMB-01 | pending -> done | worktree | child IDs, dependencies, acceptance matrix, risks, and consciously deferred scope recorded | start UMB-10A
2026-08-15 | UMB-10A | pending -> done | worktree | VM, JSONL, DAP, VS Code, format, build, Clippy, and workspace baseline evidence recorded | prepare UMB-10B contract tests
```

## Resume commands

Run from the repository root:

```powershell
git status --short --branch
git diff --stat
Get-Content docs/future/debugger/umbrella/progress.md
Get-Content docs/future/debugger/umbrella/implementation-plan.md
cargo fmt --check
cargo build --workspace --locked
```

Then run only the focused gate for the active package before the full workspace
gate. Do not clean, reset, stage, commit, merge, or push without matching user
authorization.

## Evidence log format

Append one short entry when a package changes state:

```text
YYYY-MM-DD | UMB-ID | old -> new | commit or worktree | commands/results | next exact step
```

Do not record usernames, hostnames, absolute user paths, process IDs, or other
machine-identifying data.
