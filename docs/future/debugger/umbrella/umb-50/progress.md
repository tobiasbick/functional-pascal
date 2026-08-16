# UMB-50 progress

## Current checkpoint

- Package: `UMB-50` active
- Active work IDs: none; `U50-01` is pending
- Base checkpoint: `6422489e`
- Code changes after base: documentation-only package activation
- Next action: begin `U50-01` only after an explicit continuation request
- Commit/push authorization: none for current worktree changes

## Work status

| ID | Status | Evidence or next action |
|---|---|---|
| `U50-00` | done | Format, locked workspace build, Clippy, workspace suite, VS Code host, and diff check pass at `6422489e`; stdio/host ownership inventoried |
| `U50-01` | pending | Freeze transport contracts |
| `U50-10` | pending | Proven debuggee-channel subset |
| `U50-11` | pending | Adapter/editor mapping |
| `U50-20` | pending | Live terminal I/O |
| `U50-21` | pending | Adapter/editor mapping |
| `U50-30` | pending | TUI/graph event ownership |
| `U50-31` | pending | Adapter/editor mapping |
| `U50-40` | pending | Pause-in-host feasibility |
| `U50-50` | pending | Full verification and closure |

## Baseline ownership inventory

- `fpas debug` shares process stdin/stdout between protocol and the CLI.
  Captured program output is structured `output` events. There is no separate
  live debuggee stdin.
- Hosted `Std.Console` reads block inside the intrinsic. Pause requested
  during that call is observed after it returns.
- `console.rs` (~407 LOC) and `graph/host.rs` (~391 LOC) must be split before
  adding pause-in-host or event-ownership modules.
- These are inventory facts, not acceptance of `UMB-50A`–`UMB-50D`.

## Evidence log

```text
2026-08-16 | UMB-50 | pending -> active | 6422489e base | context-loss-safe hosted-transport package created from current stdio, captured output, console/graph host, and cooperative pause-in-host evidence | execute U50-00
2026-08-16 | U50-00 | active -> done | 6422489e plus docs | format, locked workspace build, Clippy, cargo test --workspace --locked --no-fail-fast, npm test, and git diff --check pass | freeze U50-01
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
