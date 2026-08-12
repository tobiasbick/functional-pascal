# Implementation plan

Stable IDs remain valid across resumed sessions.

## Intended code layout

```text
crates/fpas-debug/src/evaluation/target.rs
  — EXISTING: confirm multi-field target preservation; add only parser regressions

crates/fpas-vm/src/vm/debug/mutation/
  mod.rs                  — MODIFY: expose prepared transition integration
  resolve.rs              — MODIFY: delegate exact qualified inactive suffixes
  transition/             — NEW: metadata resolution and complete transition preparation
    mod.rs, suffix.rs, diagnostics.rs

crates/fpas-vm/src/vm/debug/session/mutation.rs
  — MODIFY: evaluate once, prepare replacement, and reuse atomic commit

crates/fpas-vm/src/vm/debug/tests/variant_transition/
  mod.rs                  — NEW: focused verified executable fixtures
  cases.rs                — NEW: VM/session positive, negative, and edge cases

crates/fpas-debug/tests/
  variant_transition.rs       — NEW: JSONL transcript coverage
  dap_variant_transition.rs   — NEW: DAP mapping and invalidation coverage

editors/vscode/test/debugger_host/variant_transition.ts
  — NEW: real Extension Host textual transition scenario

tests/debugger/fixtures/variant_transition.fpas
  — NEW: source-compiled enum, Result, Option, nesting, and continuation fixture
```

Recheck line counts before implementation. At planning time,
`inspection/targets/payload.rs` is 475 lines and must not receive this concern;
`mutation/resolve.rs` is 246 lines and should remain a dispatcher rather than
absorbing construction logic.

## Work packages

| ID | Work package | Depends on | State | Exit gate |
|---|---|---|---|---|
| VTA-01 | Freeze qualified-target examples, errors, and executable metadata invariants | none | complete | Contract tests fail for the intended reasons before behavior changes |
| VTA-02 | Confirm target parsing preserves `root.variant.payload` without FPAS grammar changes | VTA-01 | complete | Debugger-only token tests cover casing, nested prefixes, indexes, malformed targets, and reserved language keywords |
| VTA-03 | Resolve the nearest existing writable wrapper and exact inactive variant suffix | VTA-02 | complete | Unknown, ambiguous, unqualified, empty-root, fieldless, and multi-field cases are explicit |
| VTA-04 | Validate one payload and construct one complete detached target variant | VTA-03 | complete | Enum, Result, and Option construction reuses portable metadata and existing limits |
| VTA-05 | Commit the complete wrapper through existing atomic mutation | VTA-04 | complete | Success expires handles; every failure preserves value and handles |
| VTA-06 | Add JSONL, DAP, selected-task, and VS Code mappings | VTA-05 | complete | Protocol parity and invalidation ordering pass without a new capability flag |
| VTA-07 | Update current debugger documentation and reconcile deferred scope | VTA-06 | complete | `docs/pascal/tools/` describes only verified behavior; remaining exclusions stay linked |
| VTA-08 | Run focused and full verification gates and record evidence | VTA-07 | complete | Every matrix row is PASS and `progress.md` contains exact commands |

## Dependency order

`VTA-01 -> VTA-02 -> VTA-03 -> VTA-04 -> VTA-05 -> VTA-06 -> VTA-07 -> VTA-08`

## Stop rules

- Stop at VTA-01 if a single-payload target cannot be identified without
  changing FPAS language behavior.
- Stop at VTA-03 if portable metadata cannot prove the exact owner, variant,
  field name, and field type.
- Stop at VTA-04 rather than introducing partial runtime state or default
  values.
- Stop at VTA-05 if atomic commit would require accepting an expired handle or
  weakening task/frame generation checks.
