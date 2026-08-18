# UMB-90 progress

## Current checkpoint

- Package: `UMB-90` active
- Active work ID: `U90-50`
- Base checkpoint: `b5125375`
- Code changes after base: versioned inactive-body commit; CLI-owned target
  rebuild; JSONL/DAP parity; transactional sources; VS Code commands and host test
- Next action: run full closure gates and reconcile parent evidence
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
| `U90-30` | done | Version 1 launch image; atomic inactive-body commit; function-local address remap; all workers share candidate; one previous image; compatible rollback; recording-safe reject; focused VM tests |
| `U90-31` | done | CLI rebuild provider; JSONL/DAP commit and rollback; transactional sources; DAP invalidation; VS Code commands; real Extension Host execution proves reloaded code |
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

That inventory records the pre-implementation `U90-00` checkpoint. Current
CLI-owned targets advertise reload, rebuild their exact launch input, and keep
one rollback image. Library embedders without a provider retain the frozen-off
behavior.

## Evidence log

```text
2026-08-18 | UMB-90 | pending -> active | aa2af962 base | context-loss-safe hot-reload package created from launch-owned all-stop sessions with an immutable shared executable | execute U90-00
2026-08-18 | U90-00 | active -> done | aa2af962 plus docs | format, locked workspace build, Clippy, cargo test --workspace --locked --no-fail-fast, npm test, and git diff --check pass | freeze U90-01
2026-08-18 | U90-01 | pending -> done | 1a0e6c2e | named reload rejects; hot_reload false; paired JSONL/DAP tests; cargo fmt --check, locked workspace build, Clippy, cargo test --workspace --locked --no-fail-fast, npm test, and git diff --check pass | classify U90-10
2026-08-18 | U90-10 | pending -> done | b5125375 | named accepted/rejected classes; classify-without-replace VM tests | map U90-11
2026-08-18 | U90-11 | pending -> done | b5125375 | JSONL reload.classify and DAP fpas/reloadClassify; cargo fmt --check, locked workspace build, Clippy, cargo test --workspace --locked --no-fail-fast, npm test, and git diff --check pass | reject-before-commit U90-20
2026-08-18 | U90-20 | pending -> done | worktree | replace_live_image rejects incompatibles before any image field change; accepted applied false | map U90-21
2026-08-18 | U90-21 | pending -> done | worktree | JSONL reload/image.replace and DAP fpas/reload gate; cargo fmt --check, locked workspace build, Clippy, cargo test --workspace --locked --no-fail-fast, npm test, and git diff --check pass | versioned image U90-30
2026-08-18 | U90-30 | pending -> done | worktree | versioned atomic inactive-body commit; normalized function-local metadata; address remap; breakpoint rebind; bounded rollback; recording-safe reject; VM live-image regressions pass | map U90-31
2026-08-18 | U90-31 | pending -> done | worktree | CLI exact-target rebuild provider; JSONL/DAP classify, commit, rollback, version and source parity; VS Code reload/rollback commands; npm Extension Host test executes changed inactive body; focused VM/debug tests and strict changed-library Clippy pass | close U90-50
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
