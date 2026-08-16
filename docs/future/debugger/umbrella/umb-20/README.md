# UMB-20 — Advanced breakpoint package

Status: implementation and verification complete in the worktree; recoverable
checkpoint and package cleanup remain pending.

This directory is the context-loss-safe execution package for `UMB-20` from
the parent [implementation plan](../implementation-plan.md). It covers
function breakpoints, runtime-failure filters, and the remaining bounded,
non-mutating breakpoint policy work. It does not authorize FPAS language
changes or mutating breakpoint actions.

## Resume order

1. Read this file.
2. Read [progress.md](progress.md) and perform only its next item.
3. Read [scope-and-decisions.md](scope-and-decisions.md) before changing a
   contract.
4. Locate the work item in [implementation-plan.md](implementation-plan.md).
5. Update the applicable rows in
   [verification-matrix.md](verification-matrix.md) with exact evidence.
6. Inspect the branch and worktree before editing or staging anything.

## Fixed execution rules

- Complete one work item at a time; only [progress.md](progress.md) may mark an
  item active.
- `U20-00` requires a recoverable checkpoint for the completed `UMB-10D`
  worktree. Do not commit, stage, reset, or clean without user authorization.
- Bind function breakpoints from compiler-retained function metadata and exact
  `FunctionId` values. Never infer identity from stack display strings.
- Runtime-failure filtering changes debugger stopping only. It must not make a
  failed execution resumable or hide its diagnostic and nonzero outcome.
- Implement policy in the shared Rust debugger engine. JSONL, DAP, and VS Code
  are adapters over the same state and decisions.
- Keep all limits explicit and test same-location ordering and atomic replace
  operations.
- Update current documentation only after the corresponding behavior exists.

## Completion

The package is complete when every applicable matrix row passes, parent
umbrella evidence is updated, current debugger documentation matches the
implemented adapters, and this detail package can be deleted without losing
unfinished decisions.
