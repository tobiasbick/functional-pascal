# UMB-80 progress

## Current checkpoint

- Package: `UMB-80` active
- Active work IDs: none; `U80-50` is pending
- Base checkpoint: `25138bcb`
- Code changes after base: capturing sessions reject unsupported host effects
  with `F4024` before dispatch; recording-off Random still runs; describe
  reports `replayable: false`
- Next action: begin `U80-50` only after an explicit continuation request
- Commit/push authorization: commit requested with this continuation

## Work status

| ID | Status | Evidence or next action |
|---|---|---|
| `U80-00` | done | Format, locked workspace build, Clippy, workspace suite, VS Code host, and diff check pass with `UMB-70` close; recording/replay ownership inventoried as absent |
| `U80-01` | done | Recording-off freeze; JSONL `record_replay` false; named JSONL/DAP rejects; paired tests; current debugger docs list record/replay as unsupported |
| `U80-10` | done | Versioned envelope names program and portable sources; host paths rejected without echo or mutation |
| `U80-11` | done | JSONL `recording.describe` and DAP `fpas/recordingDescribe`; no extra VS Code recording UX; record/replay stay rejected |
| `U80-20` | done | Capture is off until `start_recording`; all-stop and queued `Read`/`ReadLn` events only; in-memory ceiling 4096; no replay |
| `U80-21` | done | JSONL `record` and DAP `fpas/record`; describe reports capturing and events; replay/`stepBack` stay rejected; no extra VS Code recording UX |
| `U80-30` | done | Event ceiling advertised and enforced; overflow sets truncated; no disk files; snapshot limit 0; session-lifetime retention |
| `U80-31` | done | JSONL `recording_events`/`recording_snapshots`/`recording_disk`; describe/record report truncated and event limit; DAP camelCase parity; no extra VS Code recording UX |
| `U80-40` | done | Capturing resume rejects unsupported host effects with `F4024` before dispatch; Random is not executed |
| `U80-41` | done | JSONL/DAP `replayable: false`; capturing Random emits `F4024`; recording-off Random terminates; replay/`stepBack` stay rejected |
| `U80-50` | pending | Full verification and closure |

## Baseline ownership inventory

- JSONL `reverse_execution` and `record_replay` are false. DAP
  `supportsStepBack` is false. Envelope describe is available; capture starts
  only after `record` / `fpas/record`. Capture is bounded in session memory.
  Capturing resume rejects unsupported host effects with `F4024`. Describe
  reports `replayable: false`. No replay driver exists.
- These are inventory facts, not acceptance of `UMB-80D`.

## Evidence log

```text
2026-08-17 | UMB-80 | pending -> active | 26b47a1d base | context-loss-safe record/replay package created from launch-owned all-stop sessions with reverse_execution false | execute U80-00
2026-08-17 | U80-00 | active -> done | 26b47a1d plus docs | format, locked workspace build, Clippy, cargo test --workspace --locked --no-fail-fast, npm test, and git diff --check pass | freeze U80-01
2026-08-17 | U80-01 | pending -> done | worktree | named reverse/record rejects; record_replay false; paired JSONL/DAP tests; cargo fmt --check, locked workspace build, Clippy, cargo test --workspace --locked --no-fail-fast, npm test, and git diff --check pass | envelope U80-10
2026-08-17 | U80-10 | pending -> done | worktree | versioned envelope without host paths | map U80-11
2026-08-17 | U80-11 | pending -> done | worktree | JSONL/DAP recording.describe; cargo fmt --check, locked workspace build, Clippy, cargo test --workspace --locked --no-fail-fast, npm test, and git diff --check pass | capture U80-20
2026-08-17 | U80-20 | pending -> done | worktree | all-stop and queued-input capture; recording-off stays empty | map U80-21
2026-08-17 | U80-21 | pending -> done | worktree | JSONL/DAP record starts capture; replay stays rejected; cargo fmt --check, locked workspace build, Clippy, cargo test --workspace --locked --no-fail-fast, npm test, and git diff --check pass | bounds U80-30
2026-08-17 | U80-30 | pending -> done | worktree | advertised 4096 event ceiling; truncated; no disk or snapshot store | map U80-31
2026-08-17 | U80-31 | pending -> done | worktree | JSONL/DAP bound and truncated mapping; cargo fmt --check, locked workspace build, Clippy, cargo test --workspace --locked --no-fail-fast, npm test, and git diff --check pass | reject U80-40
2026-08-18 | U80-40 | pending -> done | worktree | capturing F4024 reject-before-dispatch; recording-off Random still terminates | map U80-41
2026-08-18 | U80-41 | pending -> done | worktree | JSONL/DAP F4024 and replayable false; cargo fmt --check, locked workspace build, Clippy, cargo test --workspace --locked --no-fail-fast, npm test, and git diff --check pass | close U80-50
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
