# UMB-80 implementation plan

## Intended code layout

```text
crates/fpas-vm/src/vm/debug/
  session.rs                   — exists: launch-owned session with capture log
  session/execution.rs         — exists: all-stop capture hooks
  session/io.rs                — exists: queued-input capture hook
  session/recording.rs         — exists: envelope describe and start_recording
  recording/
    mod.rs                     — exists: envelope and capture re-exports
    envelope.rs                — exists: versioned identity without host paths
    capture.rs                 — exists: bounded in-memory event log
  tasks.rs                     — exists: debug task runtime
crates/fpas-debug/src/
  jsonl/encode.rs              — exists: reverse_execution false, record_replay false, recording_describe true, recording_capture true, recording_disk false, recording_events 4096, recording_snapshots 0
  jsonl/server/dispatch.rs     — exists: named step_back/replay rejects; record starts capture
  jsonl/server/recording.rs    — exists: envelope, capture, truncated, and describe mapping
  dap/server.rs                — exists: supportsStepBack false
  dap/server/dispatch.rs       — exists: named stepBack/reverseContinue rejects; fpas/recordingDescribe; fpas/record
  dap/server/recording.rs      — exists: envelope, capture, and bound mapping
```

Do not add replay modules until `U80-40` is active.

## Ordered work

| ID | Status | Work | Exit gate |
|---|---|---|---|
| `U80-00` | done | Verify `UMB-70` close and current scheduler, host, and identity ownership | Recorded clean-code baseline; documentation-only transition is explicit |
| `U80-01` | done | Freeze recording contracts; inventory scheduler choices and host-visible effects | Missing effects are tests or recorded bounds |
| `U80-10` | done | Implement the proven `UMB-80A` envelope subset | Versioned identity without host paths |
| `U80-11` | done | Map accepted envelope operations through JSONL, then DAP/VS Code | Protocol parity |
| `U80-20` | done | Implement `UMB-80B` capture only after `U80-10` | Events recorded only at explicit boundaries |
| `U80-21` | done | Map capture through adapters/editor | Protocol-equivalent success and negatives |
| `U80-30` | done | Implement `UMB-80C` bounds and retention only after `U80-20` | Default paths cannot grow without bound |
| `U80-31` | done | Map bounds through adapters/editor | Capability and error parity |
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
