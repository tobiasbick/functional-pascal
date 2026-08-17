# UMB-60 implementation plan

## Intended code layout

Launch-owned protocol files are already near or above 300–400 lines. New
attach or remote behavior must enter focused modules instead of extending
those mixed roots.

```text
crates/fpas-cli/src/
  cli_debug.rs                 — exists: launch-owned JSONL/DAP stdio (~303 LOC)
crates/fpas-debug/src/
  jsonl/encode.rs              — exists: capabilities include `attach: false`
  jsonl/server.rs              — exists: launch/serve (~416 LOC)
  jsonl/server/dispatch.rs     — exists: explicit `attach` rejection
  dap/server.rs                — exists: `supportsAttach` false (~437 LOC)
  dap/server/dispatch.rs       — exists: attach/disassemble/readMemory reject
crates/fpas-debug/tests/
  attach.rs                    — exists: JSONL attach freeze and native reject
  dap_attach.rs                — exists: DAP attach freeze and native reject
editors/vscode/src/debugger/
  adapter.ts                   — exists: launch adapter; attach request rejected
```

Do not add a debuggee listener or attach handshake until `U60-10` is active.

## Ordered work

| ID | Status | Work | Exit gate |
|---|---|---|---|
| `U60-00` | done | Verify `UMB-50` close, launch-owned file sizes, and current attach=false ownership | Recorded clean-code baseline; documentation-only transition is explicit |
| `U60-01` | done | Freeze attach contracts; inventory discovery, authorization, disconnect ownership, source mapping, and native feasibility | Attach remains rejected by test; native go/no-go is `U60-30` |
| `U60-10` | pending | Implement the proven `UMB-60A` local-attach subset | Discovery, authorization, disconnect, and source mapping are deterministic |
| `U60-11` | pending | Map the accepted attach path through JSONL, then DAP/VS Code | Identity parity; no second debugger engine in the editor |
| `U60-20` | pending | Implement `UMB-60B` remote sessions only after `U60-10` | Authentication, version negotiation, recovery, and privacy limits |
| `U60-21` | pending | Map remote sessions through adapters/editor | Protocol-equivalent success and negatives |
| `U60-30` | done | Run `UMB-60C` native debugging feasibility | Rejected: native inspection would be a second semantic engine |
| `U60-40` | pending | Run full verification, reconcile docs, and checkpoint/package closure | All applicable matrix rows pass and parent evidence is complete |

## Test placement

- VM/session tests lock discovery, disconnect ownership, and source mapping
  only after a proven attach subset exists.
- JSONL and DAP tests pair attach and native-inspection failures until a
  proven attach subset exists.
- VS Code extension-host tests exercise only advertised launch UX; they do
  not duplicate VM invariants.

## Per-work-item procedure

1. Recheck branch, worktree, active ID, and named prerequisites.
2. Inspect target directory shape and line counts; record any layout change.
3. Add negative and atomicity tests first.
4. Implement the smallest shared-engine slice, then adapters.
5. Run focused format/build/tests and update [progress.md](progress.md).
6. Do not stage, commit, push, merge, or activate the next primary package
   without matching authorization and a recoverable checkpoint.
