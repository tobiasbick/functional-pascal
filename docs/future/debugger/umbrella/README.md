# Source-debugger completion umbrella

Status: `UMB-90` active; see [progress.md](progress.md) for the current
checkpoint and next executable work item.

This directory is the single resumable plan for the remaining source-debugger
backlog. It absorbs the former `DBG-D03` through `DBG-D09` rows without
duplicating them in the central backlog. The objective is to either implement
each bounded capability through the shared debugger engine or record an
evidence-backed go/no-go decision that returns a genuinely independent item to
the central backlog.

The umbrella is not permission to implement every item in one patch. Work is
performed in dependency order, one active package at a time, with a checkpoint
and acceptance gate between packages.

## Scope map

| Origin | Umbrella ownership |
|---|---|
| Former `DBG-D03` | `UMB-10`: remaining identity-bearing assignment |
| Former `DBG-D08`, low-dependency subset | `UMB-20`: function breakpoints and runtime-failure filters |
| Former `DBG-D04` | `UMB-30`: controlled lifecycle and frame changes |
| Former `DBG-D05` | `UMB-40`: task quiescence, control, and bounded history |
| Former `DBG-D06` | `UMB-50`: interactive debuggee transport and hosted programs |
| Former `DBG-D07` | `UMB-60`: attach and remote debugging; native feasibility decision |
| Former `DBG-D08`, identity-dependent subset | `UMB-70`: data breakpoints and bounded breakpoint actions |
| Former `DBG-D09`, recording subset | `UMB-80`: deterministic record and replay |
| Former `DBG-D09`, replacement subset | `UMB-90`: suspended-code hot reload |

## Source of truth

- [implementation-plan.md](implementation-plan.md) owns stable package IDs,
  dependencies, and exit gates.
- [progress.md](progress.md) owns the current checkpoint, last evidence, and
  exact resume point.
- [acceptance-matrix.md](acceptance-matrix.md) owns cross-adapter completion
  evidence.
- [architecture.md](architecture.md) owns invariants shared by every package.
- [dependency-map.md](dependency-map.md) owns ordering and parallelism rules.
- [risk-register.md](risk-register.md) owns stop conditions and mitigations.
- Completed `UMB-20` evidence is retained in parent progress, tests, current
  debugger documentation, and checkpoint `1198b1c6` rather than an obsolete
  detail plan.
- Completed `UMB-30` evidence is retained in parent progress, tests, current
  debugger documentation, and checkpoint `c2a264d0` rather than an obsolete
  detail plan.
- Completed `UMB-40` evidence is retained in parent progress, tests, current
  debugger documentation, and checkpoint `6422489e` rather than an obsolete
  detail plan.
- Completed `UMB-50` evidence is retained in parent progress, tests, current
  debugger documentation, and checkpoint `aee4f6a2` rather than an obsolete
  detail plan.
- Completed `UMB-60` evidence is retained in parent progress, tests, current
  debugger documentation, and checkpoint `eb0fbe64` rather than an obsolete
  detail plan.
- Completed `UMB-70` evidence is retained in parent progress, tests, current
  debugger documentation, and checkpoint `26b47a1d` rather than an obsolete
  detail plan.
- Completed `UMB-80` evidence is retained in parent progress, tests, current
  debugger documentation, and checkpoint `aa2af962` rather than an obsolete
  detail plan. Active `UMB-90` detail lives in [umb-90/](umb-90/).
- [consciously-deferred.md](consciously-deferred.md) records only umbrella
  boundaries, not duplicate backlog entries.
- [`../deferred.md`](../deferred.md) lists only independent work outside an
  active umbrella or the umbrella itself.

Implemented behavior remains documented only under
[`docs/pascal/tools/`](../../../pascal/tools/debugger.md). Planning claims must
not be copied into current user documentation before their acceptance gates
pass.

## Execution rules

1. Complete `UMB-00` before changing debugger behavior.
2. Mark at most one primary package `active` in [progress.md](progress.md).
   Between recoverable package checkpoints, none may be active.
3. Freeze the package contract and negative boundaries before editing code.
4. Reuse the Rust debugger engine; JSONL, DAP, and VS Code remain adapters.
5. Preserve task, frame, value, and stop-generation ownership explicitly.
6. Verify focused tests first, then the full matrix required by the package.
7. Update current user documentation only for behavior that now exists.
8. Commit one coherent package checkpoint before activating its successor.
9. Delete obsolete package detail instead of retaining implementation history.

## Resume order

After context loss:

1. Read this file.
2. Read [progress.md](progress.md).
3. Read the active package in
   [implementation-plan.md](implementation-plan.md).
4. Check its prerequisites in [dependency-map.md](dependency-map.md).
5. Check open risks in [risk-register.md](risk-register.md).
6. Inspect the current branch and worktree before trusting recorded evidence.

## Completion

The umbrella is complete only when every primary package is either:

- implemented with all applicable acceptance rows passing; or
- rejected or split by an evidence-backed decision, with only the remaining
  independent capability returned to [`../deferred.md`](../deferred.md).

The umbrella directory is then deleted. Current behavior remains in tests and
`docs/pascal/tools/`; unresolved work remains only in `deferred.md`.
