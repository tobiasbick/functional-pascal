# UMB-70 progress

## Current checkpoint

- Package: `UMB-70` active
- Active work IDs: none; `U70-01` is done
- Base checkpoint: `7ab3e705`
- Code changes after base: JSONL/DAP data-breakpoint freeze, paired rejection
  tests, current debugger docs, and VS Code launch-wiring bound
- Next action: begin `U70-10` only after an explicit continuation request
- Commit/push authorization: none for current worktree changes

## Work status

| ID | Status | Evidence or next action |
|---|---|---|
| `U70-00` | done | Format, locked workspace build, Clippy, workspace suite, VS Code host, and diff check pass with `UMB-60` close; breakpoint/mutation ownership inventoried |
| `U70-01` | done | Identity inventory frozen; `data_breakpoints`/`supportsDataBreakpoints` stay false; known commands reject without resume or mutation |
| `U70-10` | pending | Proven identity subset |
| `U70-11` | pending | Adapter/editor mapping |
| `U70-20` | pending | Data breakpoints |
| `U70-21` | pending | Adapter/editor mapping |
| `U70-30` | pending | Mutating breakpoint actions |
| `U70-31` | pending | Adapter/editor mapping |
| `U70-40` | pending | Full verification and closure |

## Baseline ownership inventory

- Source and function breakpoints exist. JSONL and DAP advertise data
  breakpoints as false and reject known commands. Capture-cell destinations
  stay rejected until `UMB-70A`.
- Inspection handles expire on resume. They are not watchpoint identities.
- These are freeze facts, not acceptance of `UMB-70A`–`UMB-70C`.

## Evidence log

```text
2026-08-17 | UMB-70 | pending -> active | eb0fbe64 base | context-loss-safe data-breakpoint package created from current source/function breakpoints and mutation identities | execute U70-00
2026-08-17 | U70-00 | active -> done | eb0fbe64 plus docs | format, locked workspace build, Clippy, cargo test --workspace --locked --no-fail-fast, npm test, and git diff --check pass | freeze U70-01
2026-08-17 | U70-01 | pending -> done | 7ab3e705 plus worktree | JSONL/DAP advertise data breakpoints false; paired rejects do not resume; capture-cell destinations stay rejected; docs name inspection IDs as stop-scoped; cargo fmt --check, git diff --check, locked workspace build, strict library Clippy, cargo test --workspace --locked --no-fail-fast, and npm test pass | wait for U70-10
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
