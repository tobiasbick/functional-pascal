# UMB-90 implementation plan

## Intended code layout

```text
crates/fpas-vm/src/vm/debug/
  live_image/                  — named classes, normalized compatibility, commit mapping
  live_image/commit.rs         — bounded prepared address remap for one atomic commit
  session.rs                   — versioned current and single previous executable
  session/live_image.rs        — classify, commit, rollback, rebind, inspection refresh
  recording/                   — exists: envelope and capture only; not a live-image store
  tasks/driver/live_image.rs   — all-worker validation and shared-image commit
crates/fpas-debug/src/
  target_reload.rs             — verified rebuilt image and source bundle
  jsonl/server/live_image.rs   — classify, commit, source swap, and rollback mapping
  dap/server/live_image.rs     — equivalent DAP mapping and invalidation
editors/vscode/src/debugger/
  liveReloadCommand.ts         — reload and rollback command surface
```

## Ordered work

| ID | Status | Work | Exit gate |
|---|---|---|---|
| `U90-00` | done | Verify `UMB-80` close and current live-image ownership | Recorded clean-code baseline; documentation-only transition is explicit |
| `U90-01` | done | Freeze hot-reload contracts; inventory compatibility surfaces | Missing rules are tests or recorded bounds |
| `U90-10` | done | Implement the proven `UMB-90A` compatibility subset | Named accepted and rejected update classes |
| `U90-11` | done | Map accepted classification through JSONL, then DAP/VS Code | Protocol parity |
| `U90-20` | done | Implement `UMB-90B` reject-before-commit only after `U90-10` | Incompatible updates leave the live image unchanged |
| `U90-21` | done | Map rejection through adapters/editor | Protocol-equivalent success and negatives |
| `U90-30` | done | Implement `UMB-90C` versioned image and rollback only after `U90-20` | Recoverable old image until commit; bounds hold |
| `U90-31` | done | Map commit and rollback through adapters/editor | Protocol-equivalent success and negatives |
| `U90-50` | pending | Run full verification, reconcile docs, and checkpoint/package closure | All applicable matrix rows pass and parent evidence is complete |

## Test placement

- VM tests lock compatibility rejects, image atomicity, and rollback.
- JSONL and DAP tests must pair the same scenario.
- VS Code extension-host tests exercise only advertised reload UX.

## Per-work-item procedure

1. Recheck branch, worktree, active ID, and named prerequisites.
2. Inspect target directory shape and line counts; record any layout change.
3. Add negative and atomicity tests first.
4. Implement the smallest shared-engine slice, then adapters.
5. Run focused format/build/tests and update [progress.md](progress.md).
6. Do not stage, commit, push, merge, or activate the next primary package
   without matching authorization and a recoverable checkpoint.
