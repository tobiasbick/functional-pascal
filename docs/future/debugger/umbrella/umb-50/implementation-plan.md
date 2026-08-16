# UMB-50 implementation plan

## Intended code layout

Protocol transport and hosted console/graph files are already near or above
300–400 lines. New behavior must enter focused modules instead of extending
those mixed roots. Split `console.rs` before adding pause-in-host work.

```text
crates/fpas-cli/src/
  cli_debug.rs                 — exists: binds JSONL/DAP to process stdio (~302 LOC)
crates/fpas-debug/src/
  jsonl/transport.rs           — exists: JSONL stdin/stdout (~84 LOC)
  jsonl/server/io.rs           — exists: `io.input` / `io.eof` / `io.cancel`
  dap/framing.rs               — exists: DAP Content-Length (~121 LOC)
  dap/server/io.rs             — exists: `fpas/input` / `fpas/eof` / `fpas/cancelInput`
crates/fpas-vm/src/vm/debug/
  io/channel.rs                — exists: session-owned debuggee channel
  session/io.rs                — exists: queued input, EOF, cancel, disconnect
  tests/transport.rs           — exists: mixing, order, EOF, quota, disconnect
  tests/events.rs              — exists: TUI/graph dispatch waits until resume
  session/events.rs            — exists: test injectors for queued TUI/graph events
crates/fpas-std/src/console/
  input.rs                     — exists: queue-only TextInput for debug
crates/fpas-vm/src/vm/hosted/
  console.rs                   — exists: Std.Console/TUI intrinsics (~407 LOC)
  graph/host.rs                — exists: Std.Graph callbacks (~391 LOC)
crates/fpas-vm/src/vm/debug/
  tests/behavior.rs            — exists: cooperative pause-in-host (~451 LOC)
editors/vscode/src/debugger/
  inputCommand.ts              — exists: send / EOF / cancel program input
```

Do not add pause-in-host modules until `U50-40`. Split `console.rs`
before adding pause-in-host work.

## Ordered work

| ID | Status | Work | Exit gate |
|---|---|---|---|
| `U50-00` | done | Verify checkpoint `6422489e`, hosted/transport file sizes, and current stdio/host ownership | Recorded clean-code baseline; documentation-only transition is explicit |
| `U50-01` | done | Freeze transport contracts; inventory protocol/debuggee mixing, live input, TUI/graph dispatch while stopped, and in-call pause | Mixing is rejected by test; live input, live terminal, and in-call pause remain current bounds |
| `U50-10` | done | Implement the proven `UMB-50A` debuggee-channel subset | Protocol bytes stay unambiguous; disconnect/EOF are deterministic |
| `U50-11` | done | Map the accepted channel through JSONL, then DAP/VS Code | Identity parity; no second console runtime in the editor |
| `U50-20` | done | Implement `UMB-50B` live terminal I/O only after `U50-10` | Ordered input, cancellation, EOF, cleanup, and output limits |
| `U50-21` | done | Map terminal I/O through adapters/editor | Protocol-equivalent success and negatives |
| `U50-30` | done | Implement the proven `UMB-50C` TUI/graph event subset | No hidden handler dispatch while stopped |
| `U50-31` | done | Map accepted event ownership through adapters/editor | Protocol-equivalent success and negatives |
| `U50-40` | pending | Run `UMB-50D` pause-in-host feasibility | Positive in-call subset or explicit rejection/dependency |
| `U50-50` | pending | Run full verification, reconcile docs, and checkpoint/package closure | All applicable matrix rows pass and parent evidence is complete |

## Test placement

- VM tests lock hosted intrinsic pause, captured output limits, and stopped
  event-dispatch invariants.
- JSONL and DAP tests must pair the same scenario and assert equivalent
  output events, input errors, and disconnect cleanup.
- VS Code extension-host tests exercise Debug Console / terminal mapping;
  they do not duplicate VM invariants.

## Per-work-item procedure

1. Recheck branch, worktree, active ID, and named prerequisites.
2. Inspect target directory shape and line counts; record any layout change.
3. Add negative and atomicity tests first.
4. Implement the smallest shared-engine slice, then adapters.
5. Run focused format/build/tests and update [progress.md](progress.md).
6. Do not stage, commit, push, merge, or activate the next primary package
   without matching authorization and a recoverable checkpoint.
