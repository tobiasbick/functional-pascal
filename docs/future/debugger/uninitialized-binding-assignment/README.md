# Uninitialized binding assignment

Status: implemented and verified on 2026-08-12; evidence is tracked in
[`progress.md`](progress.md).

This package records debugger assignment of one complete value to a visible,
source-declared mutable local or global before normal execution initializes its
storage. DAP, JSONL, and the VS Code extension use the same VM debugger engine.

## Start here

1. Read [`scope-and-decisions.md`](scope-and-decisions.md).
2. Review the data flow in [`architecture.md`](architecture.md).
3. Continue from the first incomplete item in
   [`implementation-plan.md`](implementation-plan.md).
4. Record command output and failures in [`progress.md`](progress.md).
5. Keep [`verification-matrix.md`](verification-matrix.md) synchronized with
   regressions.
6. Do not expand the boundaries in
   [`consciously-deferred.md`](consciously-deferred.md) without a new decision.

## Resume checkpoint

The implementation is present in the working tree on `codex/fpas-debugger`.
No source-language change is involved. If context is lost, inspect `git status`,
then resume at the first unchecked verification gate in `progress.md`; do not
recreate the runtime design from memory.
