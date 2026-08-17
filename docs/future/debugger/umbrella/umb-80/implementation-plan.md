# UMB-80 implementation plan

## Intended code layout

Recording files do not exist yet. New envelope, capture, or replay behavior
must enter focused modules instead of extending mixed roots.

```text
crates/fpas-vm/src/vm/debug/
  session.rs                   — exists: launch-owned session
  session/execution.rs         — exists: controlled dispatch
  tasks.rs                     — exists: debug task runtime
crates/fpas-debug/src/
  jsonl/encode.rs              — exists: reverse_execution false, record_replay false
  jsonl/server/dispatch.rs     — exists: named step_back/record/replay rejects
  dap/server.rs                — exists: supportsStepBack false
  dap/server/dispatch.rs       — exists: named stepBack/reverseContinue rejects
```

Do not add recording or replay modules until `U80-10` is active.

## Ordered work

| ID | Status | Work | Exit gate |
|---|---|---|---|
| `U80-00` | done | Verify `UMB-70` close and current scheduler, host, and identity ownership | Recorded clean-code baseline; documentation-only transition is explicit |
| `U80-01` | done | Freeze recording contracts; inventory scheduler choices and host-visible effects | Missing effects are tests or recorded bounds |
| `U80-10` | pending | Implement the proven `UMB-80A` envelope subset | Versioned identity without host paths |
| `U80-11` | pending | Map accepted envelope operations through JSONL, then DAP/VS Code | Protocol parity |
| `U80-20` | pending | Implement `UMB-80B` capture only after `U80-10` | Events recorded only at explicit boundaries |
| `U80-21` | pending | Map capture through adapters/editor | Protocol-equivalent success and negatives |
| `U80-30` | pending | Implement `UMB-80C` bounds and retention only after `U80-20` | Default paths cannot grow without bound |
| `U80-31` | pending | Map bounds through adapters/editor | Capability and error parity |
| `U80-40` | pending | Implement `UMB-80D` unsupported-effect rejection and recording-off proof | Replay claims only after rejects; disabled path unchanged |
| `U80-41` | pending | Map rejection and recording-off through adapters/editor | Protocol-equivalent success and negatives |
| `U80-50` | pending | Run full verification, reconcile docs, and checkpoint/package closure | All applicable matrix rows pass and parent evidence is complete |

## Test placement

- VM tests lock envelope identity, capture points, bounds, and recording-off.
- JSONL and DAP tests must pair the same scenario.
- VS Code extension-host tests exercise only advertised recording UX.

## Per-work-item procedure

1. Recheck branch, worktree, active ID, and named prerequisites.
2. Inspect target directory shape and line counts; record any layout change.
3. Add negative and atomicity tests first.
4. Implement the smallest shared-engine slice, then adapters.
5. Run focused format/build/tests and update [progress.md](progress.md).
6. Do not stage, commit, push, merge, or activate the next primary package
   without matching authorization and a recoverable checkpoint.
