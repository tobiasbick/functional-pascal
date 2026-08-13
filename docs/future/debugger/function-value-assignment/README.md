# Function-value assignment

Status: implemented on 2026-08-13.

This package copies an already materialized, visible, non-task-bound first-class
function value into a structurally compatible existing mutable target. The
shared VM debugger remains authoritative for JSONL, DAP, and VS Code.

This is debugger tooling only. It does not change FPAS syntax, semantics, or
the language specification.

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
