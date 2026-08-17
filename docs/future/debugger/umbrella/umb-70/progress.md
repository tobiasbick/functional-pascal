# UMB-70 progress

## Current checkpoint

- Package: `UMB-70` active
- Active work IDs: none; `U70-20` and `U70-21` are done
- Base checkpoint: `b65ecfc`
- Code changes after base: global write/change data breakpoints, JSONL/DAP
  mapping, capture-cell destinations still rejected
- Next action: begin `U70-30` only after an explicit continuation request
- Commit/push authorization: none for current worktree changes

## Work status

| ID | Status | Evidence or next action |
|---|---|---|
| `U70-00` | done | Format, locked workspace build, Clippy, workspace suite, VS Code host, and diff check pass with `UMB-60` close; breakpoint/mutation ownership inventoried |
| `U70-01` | done | Identity inventory frozen; `data_breakpoints`/`supportsDataBreakpoints` stay false; known commands reject without resume or mutation |
| `U70-10` | done | Globals keep executable slot identity; frame registers are live-activation only; capture cells stay `unregistered_alias` and reject task-bound destinations |
| `U70-11` | done | JSONL `location.describe` and DAP `fpas/locationDescribe` pair the same identities; data breakpoints stay false |
| `U70-20` | done | Global write/change watches stop; read and frame identities unverified; replace-all is atomic and bounded |
| `U70-21` | done | JSONL `data_breakpoints.replace` and DAP `dataBreakpointInfo`/`setDataBreakpoints`; `data_breakpoint.set` stays rejected; no extra VS Code UX |
| `U70-30` | pending | Mutating breakpoint actions |
| `U70-31` | pending | Adapter/editor mapping |
| `U70-40` | pending | Full verification and closure |

## Baseline ownership inventory

- Source, function, and global data breakpoints exist. JSONL and DAP advertise
  data breakpoints as true. Capture-cell destinations stay rejected; location
  describe reports `unregistered_alias`.
- Inspection handles expire on resume. Durable watchpoint identities are
  executable globals.

## Evidence log

```text
2026-08-17 | UMB-70 | pending -> active | eb0fbe64 base | context-loss-safe data-breakpoint package created from current source/function breakpoints and mutation identities | execute U70-00
2026-08-17 | U70-00 | active -> done | eb0fbe64 plus docs | format, locked workspace build, Clippy, cargo test --workspace --locked --no-fail-fast, npm test, and git diff --check pass | freeze U70-01
2026-08-17 | U70-01 | pending -> done | 7ab3e705 plus worktree | JSONL/DAP advertise data breakpoints false; paired rejects do not resume; capture-cell destinations stay rejected; docs name inspection IDs as stop-scoped; cargo fmt --check, git diff --check, locked workspace build, strict library Clippy, cargo test --workspace --locked --no-fail-fast, and npm test pass | wait for U70-10
2026-08-17 | U70-10 | pending -> done | 6f0b3b30 plus worktree | durable globals and live-frame registers; capture cells remain unregistered aliases; task-bound capture-cell destinations stay rejected | map U70-11
2026-08-17 | U70-11 | pending -> done | 6f0b3b30 plus worktree | JSONL location.describe and DAP fpas/locationDescribe; data breakpoints stay false; cargo fmt --check, git diff --check, locked workspace build, strict library Clippy, cargo test --workspace --locked --no-fail-fast, and npm test pass | wait for U70-20
2026-08-17 | U70-20 | pending -> done | b65ecfc plus worktree | global write/change data breakpoints; read and frames unverified; atomic limit; capture-cell destinations stay rejected | map U70-21
2026-08-17 | U70-21 | pending -> done | b65ecfc plus worktree | JSONL data_breakpoints.replace and DAP dataBreakpointInfo/setDataBreakpoints; cargo fmt --check, git diff --check, locked workspace build, strict library Clippy, cargo test --workspace --locked --no-fail-fast, and npm test pass | wait for U70-30
```

## Resume commands

```powershell
git status --short --branch
git diff --stat
Get-Content docs/future/debugger/umbrella/umb-70/progress.md
Get-Content docs/future/debugger/umbrella/umb-70/scope-and-decisions.md
cargo fmt --all -- --check
cargo build --workspace --locked
```

Do not clean, reset, stage, commit, push, merge, or change branches without
matching user authorization.
