# UMB-90 progress

## Current checkpoint

- Package: `UMB-90` active
- Active work IDs: none; `U90-30` is pending
- Base checkpoint: `b5125375`
- Code changes after base: reject-before-commit replace gate; JSONL/DAP
  `reload` / `image.replace` / `fpas/reload` report `applied: false`
- Next action: begin `U90-30` only after an explicit continuation request
- Commit/push authorization: commit requested with this continuation

## Work status

| ID | Status | Evidence or next action |
|---|---|---|
| `U90-00` | done | Format, locked workspace build, Clippy, workspace suite, VS Code host, and diff check pass with `UMB-80` close; live-image ownership inventoried as a shared immutable executable |
| `U90-01` | done | Hot-reload-off freeze; JSONL `hot_reload` false; named JSONL/DAP rejects; paired tests; current debugger docs list live-image reload as unsupported |
| `U90-10` | done | Proven subset accepts `unchanged` and `inactive_function_body`; named rejects cover layouts, captures, tasks, function set, anonymous closures, entry, and debug metadata; classify does not replace the live image |
| `U90-11` | done | JSONL `reload.classify` and DAP `fpas/reloadClassify` name the same classes with `applied: false`; VS Code still has no reload command |
| `U90-20` | done | `replace_live_image` rejects incompatible candidates before any `Arc<VerifiedExecutable>` change; accepted classes stay `applied: false` |
| `U90-21` | done | JSONL `reload` / `image.replace` and DAP `fpas/reload` run the gate; incompatible candidates leave the stack unchanged |
| `U90-30` | pending | Versioned live image and recoverable rollback |
| `U90-31` | pending | Adapter/editor mapping |
| `U90-50` | pending | Full verification and closure |

## Baseline ownership inventory

- `DebugSession` holds one `Arc<VerifiedExecutable>` shared with workers.
  `FunctionId` values are image-local. No committed replace-executable API exists.
- The `UMB-80` envelope names versioned portable identity. Capture is bounded
  in session memory. `recording_snapshots` is 0. Replay stays rejected.
  Hot reload must not treat that capture log as a live image.
- JSONL advertises `hot_reload: false` and `reload_classify: true`. DAP does
  not advertise hot reload. `reload` / `image.replace` / `fpas/reload` run the
  reject-before-commit gate and report `applied: false`. Classify names classes
  without applying them. VS Code exposes no reload command. `UMB-10B` remains
  blocked on this package.

## Evidence log

```text
2026-08-18 | UMB-90 | pending -> active | aa2af962 base | context-loss-safe hot-reload package created from launch-owned all-stop sessions with an immutable shared executable | execute U90-00
2026-08-18 | U90-00 | active -> done | aa2af962 plus docs | format, locked workspace build, Clippy, cargo test --workspace --locked --no-fail-fast, npm test, and git diff --check pass | freeze U90-01
2026-08-18 | U90-01 | pending -> done | 1a0e6c2e | named reload rejects; hot_reload false; paired JSONL/DAP tests; cargo fmt --check, locked workspace build, Clippy, cargo test --workspace --locked --no-fail-fast, npm test, and git diff --check pass | classify U90-10
2026-08-18 | U90-10 | pending -> done | b5125375 | named accepted/rejected classes; classify-without-replace VM tests | map U90-11
2026-08-18 | U90-11 | pending -> done | b5125375 | JSONL reload.classify and DAP fpas/reloadClassify; cargo fmt --check, locked workspace build, Clippy, cargo test --workspace --locked --no-fail-fast, npm test, and git diff --check pass | reject-before-commit U90-20
2026-08-18 | U90-20 | pending -> done | worktree | replace_live_image rejects incompatibles before any image field change; accepted applied false | map U90-21
2026-08-18 | U90-21 | pending -> done | worktree | JSONL reload/image.replace and DAP fpas/reload gate; cargo fmt --check, locked workspace build, Clippy, cargo test --workspace --locked --no-fail-fast, npm test, and git diff --check pass | versioned image U90-30
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
