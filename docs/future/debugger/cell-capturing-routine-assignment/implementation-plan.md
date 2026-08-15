# Implementation plan

Stable IDs remain valid across resumed sessions.

## Work packages

| ID | Work package | Depends on | State | Exit gate |
|---|---|---|---|---|
| CCRA-01 | Freeze fixtures, runtime-owner contract, destination boundary, diagnostics, and baseline evidence | none | planned | Positive and negative contract tests exist and fail only at the intended unsupported boundary |
| CCRA-02 | Add runtime task-owner identity to task-bound `SharedFunction` values and enforce it on ordinary call entry | CCRA-01 | planned | VM-created task-bound closures work on their owner task; foreign calls and every attempt to spawn those values fail deterministically |
| CCRA-03 | Extend exact capture-source inspection for `Cell` and `EnclosingCell` without reading payloads | CCRA-02 | planned | Binding ID, kind, type, visibility, initialization, frame, and `Arc` identity cases pass |
| CCRA-04 | Materialize a named nested routine from mixed immutable and mutable captures with bounded validation | CCRA-03 | planned | ABI order, exact sharing, task-bound flag, owner task, signature, graph, and malformed-metadata cases pass |
| CCRA-05 | Enforce same-frame frame-register destination policy and reuse one atomic commit | CCRA-04 | planned | Local/parameter register roots succeed; global, cell-backed, descendant, Dynamic, stale, and foreign roots fail without mutation |
| CCRA-06 | Prove runtime continuation, recursive-frame selection, transitive cell sharing, and cross-task containment | CCRA-05 | planned | Assigned functions share subsequent writes, selected recursion is exact, and no foreign task can invoke or spawn them |
| CCRA-07 | Prove JSONL, DAP, and VS Code parity through standard assignment surfaces | CCRA-06 | planned | Success, failure, canonical rendering, handle lifetime, invalidation, and continuation match across adapters |
| CCRA-08 | Update current debugger/editor docs and reconcile the remaining `DBG-D03` boundary | CCRA-07 | planned | `docs/pascal/tools/` describes only implemented behavior; central deferred text contains only unresolved work |
| CCRA-09 | Run focused and full verification, record exact evidence, and close every matrix row | CCRA-08 | planned | Every required row is `PASS`, or the package remains open with a named blocker |

## Dependency order

`CCRA-01 -> CCRA-02 -> CCRA-03 -> CCRA-04 -> CCRA-05 -> CCRA-06 -> CCRA-07 -> CCRA-08 -> CCRA-09`

## Detailed actions

### CCRA-01 — Contract and baseline

- Add one formatted FPAS fixture covering direct `Cell`, transitive
  `EnclosingCell`, mixed captures, shadowing, recursion, and a child-task stop.
- Record current rejection codes/messages for mutable capture assignment.
- Add failing tests for exact cell sharing and runtime foreign-task containment.
- Record focused baseline commands in [progress.md](progress.md).

### CCRA-02 — Runtime task ownership

- Extend `SharedFunction` with a runtime-only owner token and explicit
  constructors that make invalid flag/owner combinations unrepresentable or
  reject them at one boundary.
- Stamp normal VM-created mutable closures with `Worker.task_id`.
- Check task ownership before ordinary call frame entry; retain `go` rejection.
- Update equality/debug tests and every internal constructor intentionally.
- Do not serialize owner tokens in constants, units, bundles, or programs.

### CCRA-03 — Exact capture reads

- Generalize `inspection/capture_sources.rs` around recorded capture kind.
- For immutable values, retain current type and graph semantics.
- For cells, require a visible initialized cell-backed binding and clone the
  exact `Arc`; distinguish direct and enclosing kinds in diagnostics/tests.
- Keep the reader independent of display scopes and rendered summaries.

### CCRA-04 — Construction policy

- Replace the blanket mutable-cell rejection only in named-routine
  construction, not in arbitrary first-class function copying.
- Validate capture count/order/kinds, portable types, immutable subgraphs, and
  final signature before constructing a task-owned `SharedFunction`.
- Count handles under existing budgets without traversing cell payloads.
- Retain rejection of task/opaque/dynamic/nested task-bound value captures.

### CCRA-05 — Destination and commit

- Pass selected task and resolved target-root context into function preparation.
- Require request frame, lexical-owner frame, and target frame to be identical
  and owned by the selected stopped task.
- Accept only source-declared mutable frame-register roots of compatible
  function type, including an uninitialized local root when existing storage
  rules already allow complete initialization.
- Reuse the existing commit/invalidation path unchanged after preparation.

### CCRA-06 — Runtime behavior

- Prove direct and transitive captures retain pointer identity.
- Prove mutations before and after assignment are observed by original and
  debugger-created closures.
- Prove exact recursive activation selection and no peer/older-frame search.
- Prove a leaked task-owned value fails before execution on another task and
  remains rejected by task spawn.

### CCRA-07 — Adapters

- Add JSONL `variable.set` and `expression.set` coverage.
- Add DAP `setVariable` and `setExpression` parity plus negotiated invalidation.
- Add a real VS Code Extension Host case that continues execution and observes
  shared mutable state.
- Add no new command, request, or capability.

### CCRA-08 — Documentation and backlog

- Update `debugger.md`, `debugger-jsonl.md`, `debugger-dap.md`, and
  `editor-integration.md` only after behavior passes.
- Confirm normative closure/concurrency docs already describe the same language
  rule; do not change them unless implementation exposes a real discrepancy and
  the user explicitly agrees.
- Narrow `DBG-D03` only by the completed capability. Preserve every unrelated
  backlog row and all exclusions in [consciously-deferred.md](consciously-deferred.md).

### CCRA-09 — Verification

- Run formatting, Clippy, build, focused crate/protocol tests, FPAS formatter,
  full workspace tests, VS Code Extension Host tests, Markdown-link checks, and
  `git diff --check`.
- Record exact commands, results, date, and any unrelated baseline failure in
  [progress.md](progress.md); do not mark a matrix row from inspection alone.

## Exit gates

- No language syntax or semantics change.
- No serialized task owner and no display-data inference.
- Exact cell identity and selected-task ownership are proven independently.
- Arbitrary task-bound function copying remains rejected.
- One atomic commit and one generation change on success only.
- Existing immutable routine assignment and non-task-bound function copying do
  not regress.
