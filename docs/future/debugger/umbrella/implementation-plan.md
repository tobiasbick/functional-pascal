# Umbrella implementation plan

## Primary packages

| ID | Package | Depends on | Status | Exit gate |
|---|---|---|---|---|
| `UMB-00` | Checkpoint current debugger work and establish a trustworthy baseline | none | done | Worktree scope is explicit; focused gates pass; unrelated baseline failures are independently classified |
| `UMB-01` | Freeze contracts and split every inherited boundary into testable slices | `UMB-00` | done | Every child below has positive, negative, ownership, atomicity, adapter, and bound requirements or an explicit feasibility gate |
| `UMB-10` | Remaining identity-bearing assignment | `UMB-01` | done | Supported copies and compiled routine construction preserve identity; entered anonymous closures are rejected because their new executable-local identity cannot survive rollback safely |
| `UMB-20` | Function breakpoints and runtime-failure filters | `UMB-01` | done | Metadata-driven matching and equivalent stop/filter behavior pass at checkpoint `1198b1c6` |
| `UMB-30` | Controlled lifecycle and frame changes | `UMB-01` | done | Entry completion, recovery, retained-result replacement, frame restart, and initializer suppression pass in the current worktree; interior instruction changes rejected |
| `UMB-40` | Task quiescence, control, and bounded history | `UMB-30` contract | done | Deterministic task operations preserve shared-state visibility, cancellation, retention bounds, and protocol-equivalent stops |
| `UMB-50` | Interactive debuggee transport and hosted programs | `UMB-40A` | done | Protocol I/O is separated from debuggee I/O; queued terminal input, stopped TUI/graph ownership, and cooperative pause after host returns |
| `UMB-60` | Attach and remote debugging | `UMB-50` | done | Local attach, remote sessions, and native OS debugging rejected; sessions stay launch-owned |
| `UMB-70` | Data breakpoints and bounded breakpoint actions | `UMB-40A` | done | Stable data identities and mutation observation produce deterministic stops with bounded overhead and atomic actions |
| `UMB-80` | Deterministic record and replay | `UMB-40`, `UMB-50`, `UMB-70` | done | Versioned bounded capture of all-stop and queued `Read`/`ReadLn` events; unsupported host effects stop with `F4024`; recording-off unchanged; reverse execution and replay remain rejected |
| `UMB-90` | Suspended-code hot reload | `UMB-80` | done | Inactive-body commit, exact-target rebuild, source/breakpoint refresh, versioning, and bounded rollback pass across JSONL/DAP/VS Code |
| `UMB-99` | Final parity, packaging, documentation, and cleanup | all resolved packages | active | Applicable matrix rows pass; current docs match behavior; independent deferrals are centralized; umbrella plan is removed |

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
| `UMB-10B` | rejected after `UMB-90` | Enter a new anonymous closure expression | The value would retain a new image-local function ID across rollback/reload; without version-bound values and complete live-value migration, exact identity cannot be preserved |
| `UMB-10C` | done | Synthesize a bound receiver callable | Exact method identity, receiver type/layout, lifetime, and task ownership |
| `UMB-10D` | done | Dynamic endpoints, capture-cell destinations, opaque resources, and in-place callable editing | `U10D-DYN` rejected; `U10D-CELL` remains rejected after `UMB-70A` (no alias registry); `U10D-OPAQUE` rejected; `U10D-EDIT` rejected (`UMB-90` keeps code/signature); evidence retained in parent progress, tests, and current docs |

Completing one child does not mark `UMB-10` complete until every other child is
implemented or reclassified by evidence.

`UMB-10B` is not implementable as assignment-only work. `SharedFunction`
contains an executable-local `FunctionId`, and every worker resolves that ID in
the immutable `Arc<VerifiedExecutable>` shared by the session. A newly entered
body therefore requires verified code and metadata to be committed into a
versioned live image. Source-text matching against an existing function or a
debugger-only interpreter would violate the identity and one-engine contracts.
`UMB-90` deliberately accepts only a stable ordered function set. Allowing the
entered value to survive while rolling back to an image without its body would
make the stored ID dangling or version-ambiguous. Heap, frame, task, retained
result, and capture-cell migration plus version-bound callable values would be
a second live-object model disproportionate to debugger assignment. The
capability is therefore rejected rather than duplicated in the deferred list.

## `UMB-20` — Low-dependency advanced breakpoints

Completed at checkpoint `1198b1c6`. The obsolete execution detail was removed;
durable behavior and evidence remain in tests, current debugger documentation,
and [progress.md](progress.md).

| Child | Scope | Additional gate |
|---|---|---|
| `UMB-20A` | Function breakpoints | Match stable function metadata, including nested and same-named routines |
| `UMB-20B` | Rich runtime-failure filters | Filter stable diagnostic codes/categories without hiding unclassified failures |
| `UMB-20C` | Non-mutating breakpoint actions | Preserve stop ordering and bounded evaluation; mutating actions were implemented by `UMB-70C` |

## `UMB-30` — Controlled lifecycle and frame changes

Completed at checkpoint `c2a264d0`. The obsolete execution detail was removed;
durable behavior and evidence remain in tests, current debugger documentation,
and [progress.md](progress.md).

| Child | Scope | Additional gate |
|---|---|---|
| `UMB-30A` | Root and task entry completion | Typed retained result, one exit, root cancellation, no hidden execution |
| `UMB-30B` | Runtime-error recovery and retained-result replacement | Exact unconsumed failure transition; retained completed results only |
| `UMB-30C` | Frame restart and source-initializer suppression | Current args/captures retained; portable initializer identity |
| `UMB-30D` | Arbitrary instruction changes | Rejected: existing verifier dataflow cannot prove interior destinations; shared `instruction.set` / DAP `goto` rejection |

## `UMB-40` — Task quiescence, control, and history

Completed at implementation checkpoint `6422489e`. The obsolete execution
detail was removed; durable behavior and evidence remain in tests, current
debugger documentation, and [progress.md](progress.md).

| Child | Status | Scope | Additional gate |
|---|---|---|---|
| `UMB-40A` | done | Quiescence protocol | All-stop ownership, shared-state observation, blocked host work, and scheduler handoff |
| `UMB-40B` | done | Per-task pause and resume | No hidden execution of supposedly stopped peers; JSONL/DAP task identity parity |
| `UMB-40C` | done | Task creation, cancellation, and restart | Cancel stores `F4016` without command-time dispatch; create and restart rejected |
| `UMB-40D` | rejected | Non-stop execution, scheduler shortcuts, and retained history | Dirty shared-state reads, hidden shortcuts, and unbounded catalogs are forbidden |

## `UMB-50` — Interactive hosted programs

Completed at implementation checkpoint `aee4f6a2`. The obsolete execution
detail was removed; durable behavior and evidence remain in tests, current
debugger documentation, and [progress.md](progress.md).

| Child | Status | Scope | Additional gate |
|---|---|---|---|
| `UMB-50A` | done | Separate debuggee transport from JSONL/DAP | Protocol bytes stay unambiguous; disconnect/EOF are deterministic |
| `UMB-50B` | done | Live terminal input and output | Ordered queued input, cancellation, EOF, cleanup, and output limits; `live_terminal` stays false |
| `UMB-50C` | done | TUI and graph events | Handlers run only as bytecode after resume; no second editor event loop |
| `UMB-50D` | rejected | Reliable pause inside blocking host calls | In-call interruption would require splitting blocking host waits into a debug-owned wait; unsafe thread kill is forbidden. Pause remains cooperative after the intrinsic returns |

## `UMB-60` — Attach and remote

Completed at implementation checkpoint `eb0fbe64`. The obsolete execution
detail was removed; durable behavior and evidence remain in tests, current
debugger documentation, and [progress.md](progress.md).

| Child | Status | Scope | Additional gate |
|---|---|---|---|
| `UMB-60A` | rejected | Local attach to a running VM or bundle | `fpas run` uses `Vm::run` with no listener; `DebugSession` constructs the VM at launch. Connect-without-construct would be a second execution driver |
| `UMB-60B` | rejected | Remote sessions | Depends on local attach; unauthenticated remote control is forbidden |
| `UMB-60C` | rejected | OS-level native debugging | Host-process gdb/lldb would be a second semantic engine |

## `UMB-70` — Data breakpoints and actions

Completed at implementation checkpoint `26b47a1d`. The obsolete execution
detail was removed; durable behavior and evidence remain in tests, current
debugger documentation, and [progress.md](progress.md).

| Child | Status | Scope | Additional gate |
|---|---|---|---|
| `UMB-70A` | done | Stable observable data identities | Globals keep an executable slot; frame registers are live-activation only; capture cells stay `unregistered_alias` |
| `UMB-70B` | done | Data breakpoints | Global write/change watches; read and frame identities unverified; replace-all is atomic and bounded |
| `UMB-70C` | done | Mutating breakpoint actions | One global assign after condition and hit tests; function breakpoints reject assign |

## `UMB-80` — Record and replay

Completed at implementation checkpoint `aa2af962`. The obsolete execution
detail was removed; durable behavior and evidence remain in tests, current
debugger documentation, and [progress.md](progress.md).

| Child | Status | Scope | Additional gate |
|---|---|---|---|
| `UMB-80A` | done | Recording envelope and program identity | Versioned portable sources; host paths rejected without echo |
| `UMB-80B` | done | Scheduler and host-event capture | Off until `record` / `fpas/record`; all-stop and queued `Read`/`ReadLn` only |
| `UMB-80C` | done | Bounds and retention | 4,096 in-memory events; `truncated`; no disk; `recording_snapshots` 0 |
| `UMB-80D` | done | Unsupported effects and recording-off | Capturing `F4024` before dispatch; recording-off unchanged; replay stays rejected |

## `UMB-90` — Hot reload

Active. Execute only the next work ID in
[umb-90/progress.md](umb-90/progress.md). Do not replace the live executable
before compatibility and reject-before-commit are proven. Do not treat the
`UMB-80` capture log as a snapshot store.

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
