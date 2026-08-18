# UMB-90 progress

## Current checkpoint

- Package: `UMB-90` active
- Active work IDs: none; `U90-01` is pending
- Base checkpoint: `aa2af962`
- Code changes after base: documentation-only package activation plus
  `UMB-80` close
- Next action: begin `U90-01` only after an explicit continuation request
- Commit/push authorization: commit requested with this continuation

## Work status

| ID | Status | Evidence or next action |
|---|---|---|
| `U90-00` | done | Format, locked workspace build, Clippy, workspace suite, VS Code host, and diff check pass with `UMB-80` close; live-image ownership inventoried as a shared immutable executable |
| `U90-01` | pending | Freeze hot-reload contracts |
| `U90-10` | pending | Proven compatibility subset |
| `U90-11` | pending | Adapter/editor mapping |
| `U90-20` | pending | Reject incompatible updates before commit |
| `U90-21` | pending | Adapter/editor mapping |
| `U90-30` | pending | Versioned live image and recoverable rollback |
| `U90-31` | pending | Adapter/editor mapping |
| `U90-50` | pending | Full verification and closure |

## Baseline ownership inventory

- `DebugSession` holds one `Arc<VerifiedExecutable>` shared with workers.
  `FunctionId` values are image-local. No replace-executable API exists.
- The `UMB-80` envelope names versioned portable identity. Capture is bounded
  in session memory. `recording_snapshots` is 0. Replay stays rejected.
  Hot reload must not treat that capture log as a live image.
- JSONL and DAP advertise no reload command. VS Code does not expose hot-reload
  UX. `UMB-10B` remains blocked on this package.
- These are inventory facts, not acceptance of `UMB-90A`–`UMB-90C`.

## Evidence log

```text
2026-08-18 | UMB-90 | pending -> active | aa2af962 base | context-loss-safe hot-reload package created from launch-owned all-stop sessions with an immutable shared executable | execute U90-00
2026-08-18 | U90-00 | active -> done | aa2af962 plus docs | format, locked workspace build, Clippy, cargo test --workspace --locked --no-fail-fast, npm test, and git diff --check pass | freeze U90-01
```

## Resume commands

```powershell
git status --short --branch
git diff --stat
Get-Content docs/future/debugger/umbrella/umb-90/progress.md
Get-Content docs/future/debugger/umbrella/umb-90/scope-and-decisions.md
cargo fmt --all -- --check
cargo build --workspace --locked
```

Do not clean, reset, stage, commit, push, merge, or change branches without
matching user authorization.
