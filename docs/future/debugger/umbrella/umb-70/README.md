# UMB-70 data breakpoints and bounded breakpoint actions

Status: active. Resume at pending `U70-40` in [progress.md](progress.md).

This package owns stable observable data identities and mutation-observation
stops after the `UMB-40` all-stop contract. It extends the existing shared VM
debugger engine; JSONL, DAP, and VS Code remain adapters. It does not change
the FPAS language, absorb attach/remote from the rejected `UMB-60` work, or
add a second debugger engine. Capture-cell destinations (`U10D-CELL`) resume
here as `UMB-70A`.

## Files

- [scope-and-decisions.md](scope-and-decisions.md) freezes identity, data-stop,
  and mutating-action contracts.
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

`UMB-70` is complete only when each child is implemented, rejected, blocked by
a named umbrella dependency, or split by evidence; all applicable verification
rows pass; current debugger documentation describes only implemented behavior;
and a recoverable checkpoint exists. Delete this directory after that
checkpoint and retain only durable evidence in the parent plan, tests, and
current documentation.
