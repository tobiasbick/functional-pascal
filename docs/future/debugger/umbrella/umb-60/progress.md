# UMB-60 progress

## Current checkpoint

- Package: `UMB-60` active
- Active work IDs: none; `U60-01` is pending
- Base checkpoint: `aee4f6a2`
- Code changes after base: documentation-only package activation plus
  `UMB-50` close
- Next action: begin `U60-01` only after an explicit continuation request
- Commit/push authorization: none for current worktree changes

## Work status

| ID | Status | Evidence or next action |
|---|---|---|
| `U60-00` | done | Format, locked workspace build, Clippy, workspace suite, VS Code host, and diff check pass with `UMB-50` close; launch-owned attach=false inventoried |
| `U60-01` | pending | Freeze attach contracts |
| `U60-10` | pending | Proven local-attach subset |
| `U60-11` | pending | Adapter/editor mapping |
| `U60-20` | pending | Remote sessions |
| `U60-21` | pending | Adapter/editor mapping |
| `U60-30` | pending | Native debugging feasibility |
| `U60-40` | pending | Full verification and closure |

## Baseline ownership inventory

- `fpas debug` launches the debuggee. JSONL `attach` is false. DAP does not
  advertise attach. VS Code documents attach as unsupported.
- There is no discovery listener, attach handshake, or remote authentication
  surface. Protocol stdio remains the launch-owned JSONL/DAP channel.
- These are inventory facts, not acceptance of `UMB-60A`–`UMB-60C`.

## Evidence log

```text
2026-08-17 | UMB-60 | pending -> active | aee4f6a2 base | context-loss-safe attach/remote package created from launch-owned JSONL/DAP and attach:false capabilities | execute U60-00
2026-08-17 | U60-00 | active -> done | aee4f6a2 plus docs | format, locked workspace build, Clippy, cargo test --workspace --locked --no-fail-fast, npm test, and git diff --check pass | freeze U60-01
```

## Resume commands

```powershell
git status --short --branch
git diff --stat
Get-Content docs/future/debugger/umbrella/umb-60/progress.md
Get-Content docs/future/debugger/umbrella/umb-60/scope-and-decisions.md
cargo fmt --all -- --check
cargo build --workspace --locked
```

Do not clean, reset, stage, commit, push, merge, or change branches without
matching user authorization.
