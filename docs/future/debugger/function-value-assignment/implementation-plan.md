# Implementation plan

Stable IDs remain valid across resumed sessions.

## Intended code layout

```text
crates/fpas-vm/src/vm/debug/inspection/
  mod.rs                    — MODIFY: expose typed source-binding lookup internally
  handles.rs                — MODIFY: delegate shared lexical lookup
  typed_bindings.rs         — NEW: return an initialized visible value and DebugTypeId

crates/fpas-vm/src/vm/debug/mutation/
  mod.rs                    — MODIFY: expose function-value preparation
  validate.rs               — MODIFY: delegate eligible function validation
  function_value/
    mod.rs                  — NEW: source shape, runtime value, and policy orchestration
    signature.rs            — NEW: structural portable function-type compatibility
    captures.rs             — NEW: bounded capture graph safety validation

crates/fpas-vm/src/vm/debug/session/mutation.rs
  — MODIFY: resolve the target first, prepare function replacements, reuse commit

crates/fpas-vm/src/vm/debug/tests/function_value_assignment/
  mod.rs                    — NEW: focused verified executable and fixtures
  cases.rs                  — NEW: root, descendant, ownership, lifetime, and atomicity cases

crates/fpas-debug/tests/
  function_value_assignment.rs      — NEW: JSONL protocol and continuation cases
  dap_function_value_assignment.rs  — NEW: DAP parity and invalidation cases

editors/vscode/test/debugger_host/function_value_assignment.ts
  — NEW: installed Extension Host Set Value scenarios

tests/debugger/fixtures/function_value_assignment.fpas
  — NEW: source-compiled non-capturing and immutable-capture function values
```

Recheck paths and line counts immediately before implementation. At planning
time, `evaluation/execute.rs` is 473 lines, `mutation/validate.rs` is 219 lines,
and `session/mutation.rs` is 219 lines. Keep typed lookup and function policy in
their focused modules rather than growing those files into mixed concerns.

## Work packages

| ID | Work package | Depends on | State | Exit gate |
|---|---|---|---|---|
| FVA-01 | Freeze eligible source forms, signature rules, capture policy, stable errors, and source fixtures | none | complete | Contract tests fail only because function values remain rejected |
| FVA-02 | Add exact typed visible-binding lookup without changing normal expression resolution | FVA-01 | complete | Lexical shadowing, globals-only lookup, uninitialized values, stale frames, and missing metadata are covered |
| FVA-03 | Implement cycle-safe structural function signature comparison | FVA-02 | complete | Parameters, result, nested function types, record/enum identity, malformed IDs, and recursive graphs are deterministic |
| FVA-04 | Implement bounded non-executing capture eligibility validation | FVA-03 | complete | Immutable aggregates pass; cells, tasks, opaque values, task-bound functions, nested forbidden values, and limit overflow fail |
| FVA-05 | Prepare eligible SharedFunction replacements and reuse atomic commit | FVA-04 | complete | Root and existing descendant writes preserve identity; every failure preserves value and handles |
| FVA-06 | Prove JSONL, DAP, selected-task, and VS Code parity | FVA-05 | complete | Standard requests succeed consistently, failure emits no invalidation, and success emits exactly one negotiated invalidation |
| FVA-07 | Update current debugger and editor documentation and reconcile deferred scope | FVA-06 | complete | `docs/pascal/tools/` describes only implemented behavior and remaining exclusions stay linked |
| FVA-08 | Run focused and full verification gates and record evidence | FVA-07 | complete | Every matrix row is `PASS` and `progress.md` contains exact commands and dates |

## Dependency order

`FVA-01 -> FVA-02 -> FVA-03 -> FVA-04 -> FVA-05 -> FVA-06 -> FVA-07 -> FVA-08`

## Stop rules

- Stop at FVA-02 if the source binding's portable type cannot be recovered
  without changing general expression semantics or evaluating the source twice.
- Stop at FVA-03 if compatibility would depend on display names, arity alone,
  or non-portable compiler state.
- Stop at FVA-04 if safe capture eligibility requires detaching, invoking, or
  mutating the live function value.
- Stop at FVA-05 rather than permitting dynamic destinations, task-bound
  functions, stale handles, partial aggregates, or a new source-language form.
- Stop at FVA-06 if an adapter requires a custom mutation semantic instead of
  forwarding the shared VM result.

## Exit gates

- No FPAS language or grammar change.
- No new DAP capability or custom command for the standard assignment.
- No successful write without structural source and destination type proof.
- No task-bound, cell-bearing, task-bearing, or opaque-bearing source accepted.
- One atomic commit and one inspection-generation change on success only.
- Existing scalar, aggregate, variant, and uninitialized assignment suites do
  not regress.
