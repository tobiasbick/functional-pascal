# UMB-70 implementation plan

## Intended code layout

Breakpoint and mutation files already exist. New data-identity or watchpoint
behavior must enter focused modules instead of extending mixed roots.

```text
crates/fpas-vm/src/vm/debug/
  session.rs                   — exists: launch-owned session (~423 LOC)
  breakpoints/source.rs        — exists: source breakpoints (~113 LOC)
  breakpoints/function.rs      — exists: function breakpoints (~82 LOC)
  mutation/                    — exists: stopped-state writes and identities
crates/fpas-debug/src/
  breakpoints/policy.rs        — exists: stop ordering (~113 LOC)
  breakpoints/runtime_failure.rs — exists: failure filters (~114 LOC)
```

Do not add data-breakpoint modules until `U70-10` is active.

## Ordered work

| ID | Status | Work | Exit gate |
|---|---|---|---|
| `U70-00` | done | Verify `UMB-60` close, current breakpoint/mutation ownership, and `U10D-CELL` dependency | Recorded clean-code baseline; documentation-only transition is explicit |
| `U70-01` | done | Freeze identity and data-stop contracts; inventory globals, frames, cells, and descendants | Missing identities are tests or recorded bounds |
| `U70-10` | pending | Implement the proven `UMB-70A` identity subset | Exact lifetimes for supported locations; capture-cell destinations unblocked or still rejected with evidence |
| `U70-11` | pending | Map accepted identities through JSONL, then DAP/VS Code | Identity parity |
| `U70-20` | pending | Implement `UMB-70B` data breakpoints only after `U70-10` | Read/write/change stops are deterministic and bounded |
| `U70-21` | pending | Map data breakpoints through adapters/editor | Protocol-equivalent success and negatives |
| `U70-30` | pending | Implement `UMB-70C` mutating breakpoint actions only after `U70-20` | Prepare/validate/commit once; snapshots invalidate once |
| `U70-31` | pending | Map mutating actions through adapters/editor | Protocol-equivalent success and negatives |
| `U70-40` | pending | Run full verification, reconcile docs, and checkpoint/package closure | All applicable matrix rows pass and parent evidence is complete |

## Test placement

- VM tests lock identity lifetimes and data-stop ordering.
- JSONL and DAP tests must pair the same scenario.
- VS Code extension-host tests exercise only advertised watchpoint UX.

## Per-work-item procedure

1. Recheck branch, worktree, active ID, and named prerequisites.
2. Inspect target directory shape and line counts; record any layout change.
3. Add negative and atomicity tests first.
4. Implement the smallest shared-engine slice, then adapters.
5. Run focused format/build/tests and update [progress.md](progress.md).
6. Do not stage, commit, push, merge, or activate the next primary package
   without matching authorization and a recoverable checkpoint.
