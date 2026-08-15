# Umbrella implementation plan

## Primary packages

| ID | Package | Depends on | Status | Exit gate |
|---|---|---|---|---|
| `UMB-00` | Checkpoint current debugger work and establish a trustworthy baseline | none | done | Worktree scope is explicit; focused gates pass; unrelated baseline failures are independently classified |
| `UMB-01` | Freeze contracts and split every inherited boundary into testable slices | `UMB-00` | done | Every child below has positive, negative, ownership, atomicity, adapter, and bound requirements or an explicit feasibility gate |
| `UMB-10` | Remaining identity-bearing assignment | `UMB-01` | active | Each accepted value form preserves exact identity, task ownership, lifetime, type, and one-commit behavior across JSONL/DAP/VS Code |
| `UMB-20` | Function breakpoints and runtime-failure filters | `UMB-01` | pending | Stable metadata-driven matching and equivalent stop/filter behavior pass without source display inference |
| `UMB-30` | Controlled lifecycle and frame changes | `UMB-01` | pending | Accepted completion, recovery, or restart operations define cleanup, waiter effects, rollback, and selected-task behavior |
| `UMB-40` | Task quiescence, control, and bounded history | `UMB-30` contract | pending | Deterministic task operations preserve shared-state visibility, cancellation, retention bounds, and protocol-equivalent stops |
| `UMB-50` | Interactive debuggee transport and hosted programs | `UMB-40A` | pending | Protocol I/O is separated from debuggee I/O; terminal/TUI/graph events support cancellation, cleanup, and reliable pause |
| `UMB-60` | Attach and remote debugging | `UMB-50` | pending | Discovery, authentication, versions, sources, disconnect ownership, recovery, and adapter parity are proven |
| `UMB-70` | Data breakpoints and bounded breakpoint actions | `UMB-40A` | pending | Stable data identities and mutation observation produce deterministic stops with bounded overhead and atomic actions |
| `UMB-80` | Deterministic record and replay | `UMB-40`, `UMB-50`, `UMB-70` | pending | Versioned bounded recordings replay scheduler and host-visible events deterministically or reject unsupported effects |
| `UMB-90` | Suspended-code hot reload | `UMB-80` | pending | Compatibility rules cover functions, layouts, values, tasks, sources, and rollback before any live image changes |
| `UMB-99` | Final parity, packaging, documentation, and cleanup | all resolved packages | pending | Applicable matrix rows pass; current docs match behavior; independent deferrals are centralized; umbrella plan is removed |

Exactly one primary package may be marked `active` in `progress.md`.

## `UMB-00` — Checkpoint and baseline

- Inspect branch, worktree, and current CCRA implementation scope.
- Commit the completed slice only after explicit authorization.
- Re-run format, build, focused debugger tests, extension tests, and workspace
  tests.
- Investigate the current language-service reference-count mismatch separately;
  either fix its actual expectation/behavior or retain a precise baseline.
- Classify workspace Clippy findings by changed versus unrelated code.

Do not start `UMB-01` until the debugger checkpoint can be recovered without
mixing unrelated changes.

## `UMB-01` — Contract decomposition

- Freeze common diagnostic kinds, identity representations, limits, and
  adapter mapping rules.
- Add contract tests before implementation when current behavior is missing.
- Record go/no-go questions in the applicable child ID, not as new deferred
  rows.
- Update this plan if evidence changes dependencies.

## `UMB-10` — Identity-bearing assignment

| Child | Status | Scope | Additional gate |
|---|---|---|---|
| `UMB-10A` | done | Copy an already materialized task-bound function | Same-owner, same-task lifetime and escape proof; foreign, global, descendant, spawn, and stale cases fail atomically |
| `UMB-10B` | pending | Enter a new anonymous closure expression | Bounded parser and exact capture provenance; no display-text or dynamic-name inference |
| `UMB-10C` | pending | Synthesize a bound receiver callable | Exact method identity, receiver type/layout, lifetime, and task ownership |
| `UMB-10D` | pending | Dynamic endpoints, capture-cell destinations, opaque resources, and in-place callable editing | Separate feasibility decision for each identity class; no generic unsafe fallback |

Completing one child does not mark `UMB-10` complete until every other child is
implemented or reclassified by evidence.

## `UMB-20` — Low-dependency advanced breakpoints

| Child | Scope | Additional gate |
|---|---|---|
| `UMB-20A` | Function breakpoints | Match stable function metadata, including nested and same-named routines |
| `UMB-20B` | Rich runtime-failure filters | Filter stable diagnostic codes/categories without hiding unclassified failures |
| `UMB-20C` | Non-mutating breakpoint actions | Preserve stop ordering and bounded evaluation; mutating actions remain in `UMB-70C` |

## `UMB-30` — Controlled lifecycle and frame changes

| Child | Scope | Additional gate |
|---|---|---|
| `UMB-30A` | Root and task completion | Define task results, waiters, retained handles, cleanup, and terminal events |
| `UMB-30B` | Runtime-error recovery and completed-return replacement | Define resumable failure classes and prove rollback for unsupported failures |
| `UMB-30C` | Frame restart and source-initializer suppression | Reconstruct exact frame/register state without stale captures or duplicated side effects |
| `UMB-30D` | Arbitrary instruction changes | Feasibility gate for control-flow, stack, source, and verifier invariants before implementation |

## `UMB-40` — Task quiescence, control, and history

| Child | Scope | Additional gate |
|---|---|---|
| `UMB-40A` | Quiescence protocol | Define all-stop ownership, shared-state observation, blocked host work, and scheduler handoff |
| `UMB-40B` | Per-task pause and resume | No hidden execution of supposedly stopped peers; JSONL/DAP task identity parity |
| `UMB-40C` | Task creation, cancellation, and restart | Define result handles, waiters, cleanup, propagation, and deterministic errors |
| `UMB-40D` | Non-stop execution, scheduler shortcuts, and retained history | Separate bounded feasibility gates after all-stop controls are stable |

## `UMB-50` — Interactive hosted programs

| Child | Scope | Additional gate |
|---|---|---|
| `UMB-50A` | Separate debuggee transport from JSONL/DAP transport | Authenticated, recoverable lifecycle with no protocol-byte ambiguity |
| `UMB-50B` | Live terminal input and output | Ordered input, cancellation, EOF, process cleanup, and output limits |
| `UMB-50C` | Full-screen TUI and graph events | Deterministic event ownership while stopped and after resume |
| `UMB-50D` | Reliable pause inside blocking host calls | Cooperative interruption contract without unsafe thread termination |

## `UMB-60` — Attach and remote

| Child | Scope | Additional gate |
|---|---|---|
| `UMB-60A` | Local attach to a running VM or bundle | Discovery, authorization, disconnect ownership, and source mapping |
| `UMB-60B` | Remote sessions | Authentication, encryption boundary, version negotiation, recovery, and privacy limits |
| `UMB-60C` | OS-level native debugging | Go/no-go decision based on the actual runtime and bundle model; no second semantic debugger engine |

## `UMB-70` — Data breakpoints and actions

| Child | Scope | Additional gate |
|---|---|---|
| `UMB-70A` | Stable observable data identities | Globals, frame registers, cells, and supported descendants have exact lifetimes |
| `UMB-70B` | Data breakpoints | Read/write/change semantics and bounded overhead are deterministic across tasks |
| `UMB-70C` | Mutating breakpoint actions | Reuse prepare/validate/commit and invalidate snapshots exactly once |

## `UMB-80` — Record and replay

- Version the recording envelope and source/program identity.
- Record scheduler choices and supported host inputs at explicit boundaries.
- Bound memory, disk, event count, snapshot count, and retention.
- Reject unsupported nondeterminism before claiming replayability.
- Prove forward execution is unchanged when recording is disabled.

## `UMB-90` — Hot reload

- Define compatibility for active and inactive function bodies.
- Define record, enum, global, closure, task, and debug-metadata migration.
- Reject incompatible updates before changing the live program image.
- Preserve a recoverable old image until the new image commits.
- Prove JSONL/DAP/VS Code report the same accepted and rejected changes.

## `UMB-99` — Closure

- Run every applicable row in [acceptance-matrix.md](acceptance-matrix.md).
- Reconcile `docs/pascal/tools/` with implemented behavior and remove obsolete
  limitations.
- Produce and smoke-test a local VSIX when editor behavior changed.
- Move only independent unresolved capabilities back to `../deferred.md`.
- Delete this directory after the final implementation checkpoint is committed.
