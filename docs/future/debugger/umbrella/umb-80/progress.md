# UMB-80 progress

## Current checkpoint

- Package: `UMB-80` active
- Active work IDs: none; `U80-10` is pending
- Base checkpoint: `85c420f7`
- Code changes after base: recording-off freeze and named reverse/record
  rejects in JSONL and DAP
- Next action: begin `U80-10` only after an explicit continuation request
- Commit/push authorization: commit requested with this continuation

## Work status

| ID | Status | Evidence or next action |
|---|---|---|
| `U80-00` | done | Format, locked workspace build, Clippy, workspace suite, VS Code host, and diff check pass with `UMB-70` close; recording/replay ownership inventoried as absent |
| `U80-01` | done | Recording-off freeze; JSONL `record_replay` false; named JSONL/DAP rejects; paired tests; current debugger docs list record/replay as unsupported |
| `U80-10` | pending | Proven envelope subset |
| `U80-11` | pending | Adapter/editor mapping |
| `U80-20` | pending | Scheduler and host-event capture |
| `U80-21` | pending | Adapter/editor mapping |
| `U80-30` | pending | Bounds and retention |
| `U80-31` | pending | Adapter/editor mapping |
| `U80-40` | pending | Unsupported-effect rejection and recording-off proof |
| `U80-41` | pending | Adapter/editor mapping |
| `U80-50` | pending | Full verification and closure |

## Baseline ownership inventory

- JSONL `reverse_execution` and `record_replay` are false. DAP
  `supportsStepBack` is false. No recording envelope or replay driver exists.
- These are inventory facts, not acceptance of `UMB-80A`–`UMB-80D`.

## Evidence log

```text
2026-08-17 | UMB-80 | pending -> active | 26b47a1d base | context-loss-safe record/replay package created from launch-owned all-stop sessions with reverse_execution false | execute U80-00
2026-08-17 | U80-00 | active -> done | 26b47a1d plus docs | format, locked workspace build, Clippy, cargo test --workspace --locked --no-fail-fast, npm test, and git diff --check pass | freeze U80-01
2026-08-17 | U80-01 | pending -> done | worktree | named reverse/record rejects; record_replay false; paired JSONL/DAP tests; cargo fmt --check, locked workspace build, Clippy, cargo test --workspace --locked --no-fail-fast, npm test, and git diff --check pass | envelope U80-10
```

## Resume commands

```powershell
git status --short --branch
git diff --stat
Get-Content docs/future/debugger/umbrella/umb-80/progress.md
Get-Content docs/future/debugger/umbrella/umb-80/scope-and-decisions.md
cargo fmt --all -- --check
cargo build --workspace --locked
```

Do not clean, reset, stage, commit, push, merge, or change branches without
matching user authorization.
