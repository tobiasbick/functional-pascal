# Verification matrix

Status values are `PLANNED`, `PASS`, or `BLOCKED`.

| ID | Acceptance case | Planned evidence | Status |
|---|---|---|---|
| FVA-T01 | A visible non-capturing function value replaces a compatible mutable local | VM and source fixture | PASS |
| FVA-T02 | An immutable-capture closure is copied without reconstruction, and a mutable capture-cell source is materialized before validation | VM identity, capture-source, and continuation assertions | PASS |
| FVA-T03 | Compatible uninitialized mutable local and global roots accept one complete function value | VM and JSONL cases | PASS |
| FVA-T04 | Mutable parameter, mutable capture cell, global, record field, array element, existing dictionary value, and active payload destinations retain current ownership rules | VM table-driven cases | PASS |
| FVA-T05 | Source lookup follows lexical shadowing and frame selection; omitted frame searches globals only | typed-binding unit and session cases | PASS |
| FVA-T06 | Parameter-count, parameter-type, result-type, nested function-type, record-owner, and enum-owner mismatches are rejected | structural signature unit cases | PASS |
| FVA-T07 | Missing, uninitialized, dynamic, non-function, and metadata-less source bindings are rejected actionably | VM, JSONL, and DAP negative cases | PASS |
| FVA-T08 | Task-bound closures are rejected for local, capture, descendant, and global destinations | VM policy cases | PASS |
| FVA-T09 | Direct or nested cells, task handles, opaque handles, and task-bound functions in captures are rejected | bounded capture graph unit cases | PASS |
| FVA-T10 | Oversized, deeply nested, shared, and adversarial capture graphs terminate within limits | limit and shared-node unit cases | PASS |
| FVA-T11 | Direct named routines, bound methods, calls returning functions, new closure syntax, and aggregate function sources remain rejected | parser/session/protocol negative cases | PASS |
| FVA-T12 | Inactive-variant function payload transition and dynamic destination remain rejected | VM and protocol negative cases | PASS |
| FVA-T13 | Immutable, hidden, stale, foreign-task, unavailable, and unsupported targets retain existing errors | VM session regressions | PASS |
| FVA-T14 | Failure preserves the old function identity, live value, stop state, and inspection references | VM atomicity and protocol transcript cases | PASS |
| FVA-T15 | Success expires old references once, refreshes every stopped snapshot, and continuation invokes the replacement | VM, JSONL, DAP, and source fixture | PASS |
| FVA-T16 | JSONL and DAP return equivalent values, type names, error codes, and hints | paired transcripts | PASS |
| FVA-T17 | VS Code Variables and Watch Set Value forward standard requests and refresh correctly | Extension Host test | PASS |
| FVA-T18 | Existing call evaluation can invoke the copied value under unchanged detached-call policy | source and protocol continuation cases | PASS |
| FVA-T19 | Existing scalar, aggregate, variant, uninitialized, dictionary, and sequence mutation do not regress; selectors and replacement retain one shared budget | focused existing suites and exact operation-boundary assertions | PASS |
| FVA-T20 | FPAS fixture formatting is stable | `fpas fmt --check tests/debugger/fixtures/function_value_assignment.fpas` | PASS |
| FVA-T21 | Rust formatting, lint, build, workspace tests, and VS Code tests pass | commands recorded in `progress.md` | PASS |

## Required protocol assertions

For success, assert the rendered function summary, selected task and frame,
continuation result, old-handle expiry, refreshed value, and exactly one DAP
invalidation when negotiated. For failure, assert stable code and hint,
unchanged callable identity and continuation behavior, preserved handles,
stopped state, and absence of invalidation.
