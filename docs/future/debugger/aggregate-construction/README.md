# Explicit aggregate construction

Status: implemented on 2026-08-13.

This package adds one explicit, metadata-driven way to inspect and atomically
construct complete enum, `Result`, and `Option` variants while the debugger is
stopped. The same VM operation is exposed through JSONL, DAP custom requests,
and the VS Code command **Debug: Construct Variant**.

This is debugger tooling only. It does not change FPAS syntax, semantics, or
the language specification. Existing constructor assignment through
`setVariable` / `setExpression` remains supported.

Current user-facing behavior is documented in
[`docs/pascal/tools/debugger.md`](../../../pascal/tools/debugger.md). Remaining
exclusions stay in [`consciously-deferred.md`](consciously-deferred.md) and
[`docs/future/debugger/deferred.md`](../deferred.md).

## Record

1. Accepted scope and decisions: [`scope-and-decisions.md`](scope-and-decisions.md).
2. Ownership boundaries: [`architecture.md`](architecture.md).
3. Completed work packages: [`implementation-plan.md`](implementation-plan.md).
4. Verification rows: [`verification-matrix.md`](verification-matrix.md).
5. Commands and dates: [`progress.md`](progress.md).
