# UMB-90 suspended-code hot reload

Status: active. Resume at pending `U90-01` in [progress.md](progress.md).

This package owns compatibility proofs and versioned live-image replacement
for a stopped debug session after the `UMB-80` recording identity and bound
capture contracts. It extends the existing shared VM debugger engine; JSONL,
DAP, and VS Code remain adapters. It does not change the FPAS language, reopen
attach/remote from the rejected `UMB-60` work, add a second debugger engine, or
enable recording replay. Recording snapshots stay at zero; hot reload must not
treat the capture log as a live image.

## Files

- [scope-and-decisions.md](scope-and-decisions.md) freezes compatibility,
  reject-before-commit, and rollback contracts.
- [implementation-plan.md](implementation-plan.md) owns ordered work IDs and
  the intended source layout.
- [verification-matrix.md](verification-matrix.md) defines required positive,
  negative, adapter, bound, and full-suite evidence.
- [progress.md](progress.md) is the only child checkpoint to update while this
  package is active.

## Resume order

1. Inspect the branch and worktree; do not assume the recorded checkpoint is
   still clean.
2. Read [progress.md](progress.md) and execute only its next work ID.
3. Re-read the applicable contract in
   [scope-and-decisions.md](scope-and-decisions.md).
4. Add negative and atomicity tests before enabling a new successful path.
5. Record exact commands and results in [progress.md](progress.md).

## Package exit

`UMB-90` is complete only when each child is implemented, rejected, blocked by
a named umbrella dependency, or split by evidence; all applicable verification
rows pass; current debugger documentation describes only implemented behavior;
and a recoverable checkpoint exists. Delete this directory after that
checkpoint and retain only durable evidence in the parent plan, tests, and
current documentation.
