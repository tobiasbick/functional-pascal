# Progress

Last updated: 2026-08-13

## Current checkpoint

Implemented on 2026-08-13. The bounded function-value copy slice is live on the
shared VM debugger and the JSONL, DAP, and VS Code surfaces. Remaining
exclusions stay in `consciously-deferred.md`.

The implementation review was closed by routing named function sources through
the ordinary evaluator, which materializes mutable capture cells and charges
the source lookup to the same operation budget as textual selectors. Core DAP
failures now retain the JSONL error code and help text, with a paired protocol
transcript asserting value, type, code, message, and hint parity.

## Work-package status

| ID | State | Evidence or next action |
|---|---|---|
| FVA-01 | complete | Source fixture plus VM/protocol negative cases for ineligible sources |
| FVA-02 | complete | `typed_bindings.rs` and lexical/globals-only session cases |
| FVA-03 | complete | `signature.rs` unit tests for structure, layout identity, cycles, and malformed IDs |
| FVA-04 | complete | `captures.rs` unit tests for aggregates, forbidden values, nesting, sharing, and limits |
| FVA-05 | complete | Prepare after destination resolve; atomic commit; identity and failure-preservation cases |
| FVA-06 | complete | JSONL, DAP invalidation, selected-child-task, and Extension Host Set Value |
| FVA-07 | complete | Current `docs/pascal/tools/` pages and editor README describe the implemented copy |
| FVA-08 | complete | Commands below; every `verification-matrix.md` row is `PASS` |

## Evidence log

Recorded 2026-08-13:

```text
cargo fmt --all -- --check
# exit 0

cargo clippy -p fpas-vm -p fpas-debug --all-features --locked -- -D warnings
# exit 0

cargo build
# exit 0

cargo test --workspace --no-fail-fast
# exit 0 (after restoring shared selector/replacement evaluation budget)

cargo test -p fpas-vm --lib function_value
# 23 passed, including mutable capture-source materialization and shared-budget boundaries

cargo test -p fpas-debug --test function_value_assignment
# 2 passed, including selected-child-task binding

cargo test -p fpas-debug --test dap_function_value_assignment
# 3 passed, including paired JSONL/DAP result and error metadata

cargo test -p fpas-debug --test variable_mutation textual_selectors_share_one_call
# 1 passed

cargo run -p fpas-cli -- fmt --check tests/debugger/fixtures/function_value_assignment.fpas
# exit 0

cd editors/vscode && npm test
# exit 0 with a statement boundary between the initialized marker and the mutated call

git diff --check
# exit 0 (CRLF conversion warnings only)
```

Focused mapping:

- FVA-T01..T05, T07, T08, T11..T15, T18: `crates/fpas-vm/src/vm/debug/tests/function_value_assignment/`
- FVA-T06, T09, T10: `mutation/function_value/signature.rs` and `captures.rs` unit tests
- FVA-T03, T07, T11, T14..T16, T18: JSONL and DAP protocol tests plus the compiled fixture
- FVA-T17: `editors/vscode/test/debugger_host/function_value_assignment.ts`
- FVA-T19: workspace suite, including restored shared-budget `variable_mutation` coverage
- FVA-T20, T21: commands above

## Resume instructions

The slice is implemented. Further function-value work belongs in a new package
that starts from `consciously-deferred.md` rather than reopening these work
IDs. Do not mark a matrix row `PASS` from inspection alone if behavior changes
again.
