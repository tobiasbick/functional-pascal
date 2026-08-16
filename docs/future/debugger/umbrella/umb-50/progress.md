# UMB-50 progress

## Current checkpoint

- Package: `UMB-50` active
- Active work IDs: none; `U50-20` is pending
- Base checkpoint: `6422489e`
- Code changes after base: debuggee channel, live-input rejection, JSONL/DAP/VS Code mapping
- Next action: begin `U50-20` only after an explicit continuation request
- Commit/push authorization: none for current worktree changes

## Work status

| ID | Status | Evidence or next action |
|---|---|---|
| `U50-00` | done | Format, locked workspace build, Clippy, workspace suite, VS Code host, and diff check pass at `6422489e`; stdio/host ownership inventoried |
| `U50-01` | done | Mixing rejected: raw JSONL/DAP stdin is a protocol error; `WriteLn` is structured `output` only |
| `U50-10` | done | Session-owned channel connects at launch, closes on disconnect without dispatch, rejects live input atomically |
| `U50-11` | done | JSONL `io.input`, DAP `fpas/input`, capabilities `live_input`/`live_terminal` false, VS Code host has no second console |
| `U50-20` | pending | Live terminal I/O |
| `U50-21` | pending | Adapter/editor mapping |
| `U50-30` | pending | TUI/graph event ownership |
| `U50-31` | pending | Adapter/editor mapping |
| `U50-40` | pending | Pause-in-host feasibility |
| `U50-50` | pending | Full verification and closure |

## Frozen bounds

- Protocol stdin EOF ends `serve`; it is not debuggee stdin EOF.
- TUI/graph handlers run only as bytecode inside hosted intrinsics. Pending OS
  events while stopped belong to `U50-30`.
- Pause during a blocking host intrinsic is observed after the intrinsic
  returns. In-call interruption belongs to `U50-40`.
- `console.rs` and `graph/host.rs` still need splits before pause-in-host or
  event-ownership modules.

## Evidence log

```text
2026-08-16 | UMB-50 | pending -> active | 6422489e base | context-loss-safe hosted-transport package created from current stdio, captured output, console/graph host, and cooperative pause-in-host evidence | execute U50-00
2026-08-16 | U50-00 | active -> done | 6422489e plus docs | format, locked workspace build, Clippy, cargo test --workspace --locked --no-fail-fast, npm test, and git diff --check pass | freeze U50-01
2026-08-16 | U50-01 | pending -> done | 6422489e plus worktree | raw protocol stdin rejected; structured output only; live input, live terminal, and in-call pause recorded as bounds | implement U50-10
2026-08-16 | U50-10 | pending -> done | 6422489e plus worktree | session channel connect/close; live input rejects without mutation | map adapters
2026-08-16 | U50-11 | pending -> done | 6422489e plus worktree | JSONL `io.input`, DAP `fpas/input`, VS Code host structured output, no second console; cargo fmt, locked workspace build/tests, Clippy, and npm test pass | wait before U50-20
```

## Resume commands

```powershell
git status --short --branch
git diff --stat
Get-Content docs/future/debugger/umbrella/umb-50/progress.md
Get-Content docs/future/debugger/umbrella/umb-50/scope-and-decisions.md
cargo fmt --all -- --check
cargo build --workspace --locked
```

Do not clean, reset, stage, commit, push, merge, or change branches without
matching user authorization.
