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
  dap/server.rs                — exists: launch initialize; no attach request (~437 LOC)
editors/vscode/src/debugger/
  adapter.ts                   — exists: launch adapter (~105 LOC)
```

Do not add attach or remote modules until `U60-10` is active.

## Ordered work

| ID | Status | Work | Exit gate |
|---|---|---|---|
| `U60-00` | done | Verify `UMB-50` close, launch-owned file sizes, and current attach=false ownership | Recorded clean-code baseline; documentation-only transition is explicit |
| `U60-01` | pending | Freeze attach contracts; inventory discovery, authorization, disconnect ownership, source mapping, and native feasibility | Attach remains rejected by test or recorded as the current bound |
| `U60-10` | pending | Implement the proven `UMB-60A` local-attach subset | Discovery, authorization, disconnect, and source mapping are deterministic |
| `U60-11` | pending | Map the accepted attach path through JSONL, then DAP/VS Code | Identity parity; no second debugger engine in the editor |
| `U60-20` | pending | Implement `UMB-60B` remote sessions only after `U60-10` | Authentication, version negotiation, recovery, and privacy limits |
| `U60-21` | pending | Map remote sessions through adapters/editor | Protocol-equivalent success and negatives |
| `U60-30` | pending | Run `UMB-60C` native debugging feasibility | Positive native subset or explicit rejection |
| `U60-40` | pending | Run full verification, reconcile docs, and checkpoint/package closure | All applicable matrix rows pass and parent evidence is complete |

## Test placement

- VM/session tests lock discovery, disconnect ownership, and source mapping.
- JSONL and DAP tests must pair the same scenario and assert equivalent
  attach failures until a proven subset exists.
- VS Code extension-host tests exercise only advertised attach UX; they do
  not duplicate VM invariants.

## Per-work-item procedure

1. Recheck branch, worktree, active ID, and named prerequisites.
2. Inspect target directory shape and line counts; record any layout change.
3. Add negative and atomicity tests first.
4. Implement the smallest shared-engine slice, then adapters.
5. Run focused format/build/tests and update [progress.md](progress.md).
6. Do not stage, commit, push, merge, or activate the next primary package
   without matching authorization and a recoverable checkpoint.
