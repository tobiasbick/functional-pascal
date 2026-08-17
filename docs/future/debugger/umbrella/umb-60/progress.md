# UMB-60 progress

## Current checkpoint

- Package: `UMB-60` active
- Active work IDs: none; `U60-10` is pending
- Base checkpoint: `aee4f6a2`
- Code changes after base: attach/native rejection freeze
- Next action: begin `U60-10` only after an explicit continuation request
- Commit/push authorization: none for current worktree changes

## Work status

| ID | Status | Evidence or next action |
|---|---|---|
| `U60-00` | done | Format, locked workspace build, Clippy, workspace suite, VS Code host, and diff check pass with `UMB-50` close; launch-owned attach=false inventoried |
| `U60-01` | done | JSONL/DAP attach rejected without launch or resume; VS Code rejects `request: attach`; no discovery listener |
| `U60-10` | pending | Proven local-attach subset |
| `U60-11` | pending | Adapter/editor mapping |
| `U60-20` | pending | Remote sessions |
| `U60-21` | pending | Adapter/editor mapping |
| `U60-30` | done | Native inspection rejected; disassemble/memory/registers stay unsupported |
| `U60-40` | pending | Full verification and closure |

## Frozen bounds

- The debugger constructs an in-process VM at launch. Attach remains false
  until a debuggee-owned listener and connect-without-construct path exist.
- Native OS debugging of the host process is rejected. One engine inspects
  FPAS source and bytecode only.
- `live_terminal` remains false. Protocol stdin is never debuggee stdin.

## Evidence log

```text
2026-08-17 | UMB-60 | pending -> active | aee4f6a2 base | context-loss-safe attach/remote package created from launch-owned JSONL/DAP and attach:false capabilities | execute U60-00
2026-08-17 | U60-00 | active -> done | aee4f6a2 plus docs | format, locked workspace build, Clippy, cargo test --workspace --locked --no-fail-fast, npm test, and git diff --check pass | freeze U60-01
2026-08-17 | U60-01 | pending -> done | fb91a7c7 plus worktree | JSONL/DAP attach reject without mutation; VS Code attach request rejected; current docs | wait before U60-10
2026-08-17 | U60-30 | pending -> done | fb91a7c7 plus worktree | native disassemble/memory/registers unsupported; second semantic engine forbidden | wait before U60-10
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
