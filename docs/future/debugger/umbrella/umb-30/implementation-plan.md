# UMB-30 implementation plan

## Intended code layout

The current session and adapter roots are already about 430-440 lines, and the
task driver is about 375 lines. New behavior must enter focused modules instead
of extending those mixed roots.

```text
crates/fpas-vm/src/vm/debug/
  lifecycle/
    mod.rs                 — NEW: lifecycle exports and shared result types
    completion.rs          — NEW: root/task entry completion preparation
    recovery.rs            — NEW: failed-stop recovery preparation
    completed_result.rs    — NEW if accepted: retained-result replacement
    restart.rs             — NEW: frame reconstruction
    instruction.rs         — NEW only after U30-50 feasibility passes
  session/
    lifecycle.rs           — NEW: stop/frame ownership and one-commit facade
  tasks/
    completion.rs          — NEW if driver ownership would exceed its focus
crates/fpas-vm/src/vm/tasks/
  scheduler.rs             — MODIFY: delegate exact result transitions
  scheduler/
    results.rs             — NEW if result-transition logic materially grows
crates/fpas-debug/src/jsonl/server/
  lifecycle.rs             — NEW: JSONL command mapping
crates/fpas-debug/src/dap/server/
  lifecycle.rs             — NEW: DAP/custom-request mapping
editors/vscode/src/
  debuggerLifecycle.ts     — NEW only if editor-owned UI mapping is needed
```

Compiler, bytecode, unit, linker, and bundle metadata files may be modified
only if `U30-41` proves source-initializer identity requires portable metadata.
Record their exact layout before editing them.

## Ordered work

| ID | Status | Work | Exit gate |
|---|---|---|---|
| `U30-00` | done | Verify checkpoint `1198b1c6`, focused lifecycle baseline, file sizes, and current runtime/scheduler ownership | Recorded clean-code baseline; documentation-only transition is explicit |
| `U30-01` | done | Freeze command/error/event contracts; inventory scheduler result identity and source-binding metadata; add rejection/atomicity tests | Entry transition is frozen; recovery and initializer metadata gaps are recorded explicitly |
| `U30-10` | done | Implement shared root/task entry-frame completion | Typed retained result, one exit, root cancellation, no hidden execution, and atomic rejection pass in VM tests |
| `U30-11` | done | Map entry completion through JSONL, DAP, and VS Code; update current docs | JSONL/DAP parity and the real extension-host lifecycle test pass |
| `U30-20` | done | Implement exact runtime-error recovery for live and entry frames | Exact unconsumed failure transition, preserved diagnostic, and stale/consumed rejection pass |
| `U30-21` | done | Map recovery through adapters/editor and document it | Protocol-equivalent recovery and negative errors pass |
| `U30-30` | done | Decide and implement the provable completed-return replacement subset | Retained-result replacement passes without overclaiming removed ordinary call frames |
| `U30-40` | done | Implement selected live-frame restart | Current args/captures retained; locals/temporaries reset; younger frames removed; no command-time execution |
| `U30-41` | done | Prove and implement source-initializer suppression or record the missing portable identity | Exact metadata round trip and suppression tests pass, or precise dependency is recorded |
| `U30-42` | done | Map restart/suppression through adapters/editor and current docs | Restart and suppression parity pass through existing mutation commands |
| `U30-50` | pending | Run arbitrary instruction-change feasibility; implement only destinations whose invariants are provable | Positive safe-destination and negative CFG/dataflow tests pass, or explicit rejection/dependency is recorded |
| `U30-60` | pending | Run full verification, reconcile docs, and checkpoint/package closure | All applicable matrix rows pass and parent evidence is complete |

## Test placement

- VM lifecycle tests belong beside the new lifecycle modules or in focused
  integration files named for completion, recovery, restart, and instruction
  changes.
- JSONL and DAP tests must pair the same scenario and assert equivalent stable
  error codes, events, and resulting state.
- VS Code extension-host tests exercise editor commands and standard DAP state;
  they do not duplicate VM invariants.
- Portable metadata, if required, needs bytecode encode/decode, unit, linker,
  program/bundle, and malformed-artifact coverage.

## Per-work-item procedure

1. Recheck branch, worktree, active ID, and named prerequisites.
2. Inspect target directory shape and line counts; record any layout change.
3. Add negative and atomicity tests first.
4. Implement the smallest shared-engine slice, then adapters.
5. Run focused format/build/tests and update [progress.md](progress.md).
6. Do not stage, commit, push, merge, or activate the next primary package
   without matching authorization and a recoverable checkpoint.
