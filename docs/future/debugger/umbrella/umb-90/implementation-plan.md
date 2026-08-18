# UMB-90 implementation plan

## Intended code layout

Reload files do not exist yet. New compatibility, image, or rollback behavior
must enter focused modules instead of extending mixed roots.

```text
crates/fpas-vm/src/vm/debug/
  session.rs                   — exists: launch-owned session; shared Arc<VerifiedExecutable>
  recording/                   — exists: envelope and capture only; not a live-image store
  tasks.rs                     — exists: debug task runtime
crates/fpas-debug/src/
  jsonl/encode.rs              — exists: hot_reload false
  jsonl/server/dispatch.rs     — exists: named reload / image.replace rejects
  dap/server.rs                — exists: no hot-reload advertisement
  dap/server/dispatch.rs       — exists: named fpas/reload reject
```

Do not add reload modules until `U90-10` is active.

## Ordered work

| ID | Status | Work | Exit gate |
|---|---|---|---|
| `U90-00` | done | Verify `UMB-80` close and current live-image ownership | Recorded clean-code baseline; documentation-only transition is explicit |
| `U90-01` | done | Freeze hot-reload contracts; inventory compatibility surfaces | Missing rules are tests or recorded bounds |
| `U90-10` | pending | Implement the proven `UMB-90A` compatibility subset | Named accepted and rejected update classes |
| `U90-11` | pending | Map accepted classification through JSONL, then DAP/VS Code | Protocol parity |
| `U90-20` | pending | Implement `UMB-90B` reject-before-commit only after `U90-10` | Incompatible updates leave the live image unchanged |
| `U90-21` | pending | Map rejection through adapters/editor | Protocol-equivalent success and negatives |
| `U90-30` | pending | Implement `UMB-90C` versioned image and rollback only after `U90-20` | Recoverable old image until commit; bounds hold |
| `U90-31` | pending | Map commit and rollback through adapters/editor | Protocol-equivalent success and negatives |
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
