# UMB-20 implementation plan

Work items are ordered. Do not start an item whose prerequisites are not done.

## Intended file layout

```text
crates/fpas-vm/src/vm/debug/
  breakpoints.rs                         — SPLIT: focused module root
  breakpoints/
    source.rs                            — MOVED: source binding
    function.rs                          — NEW: FunctionId and entry binding
  session.rs                             — SPLIT: remove breakpoint ownership methods
  session/breakpoints.rs                 — NEW: atomic source/function collections
  session/execution.rs                   — MODIFY: deterministic shared-boundary stops
  tests/breakpoints/
    function.rs                          — NEW: identity, collisions, recursion, tasks
    ordering.rs                          — NEW: shared-boundary policy ordering
crates/fpas-debug/src/
  breakpoints/policy.rs                  — MODIFY: breakpoint-kind-neutral policy
  breakpoints/runtime_failure.rs         — NEW: exact code/filter policy
  jsonl/server/breakpoints.rs            — SPLIT when needed; keep a focused module root
  jsonl/server/breakpoints/function.rs   — NEW: function commands
  jsonl/server/runtime_failures.rs        — NEW: filter command and completion mapping
  jsonl/server/completion.rs              — MODIFY: filtered failure termination
  dap/server.rs                           — SPLIT before adding behavior
  dap/server/breakpoints.rs               — NEW/MOVED: source and function requests
  dap/server/exceptions.rs                — NEW: exception filters and mapping
crates/fpas-debug/tests/
  function_breakpoints.rs                — NEW: JSONL integration
  dap_function_breakpoints.rs            — NEW: DAP integration
  runtime_failure_filters.rs             — NEW: JSONL integration
  dap_runtime_failure_filters.rs         — NEW: DAP integration
editors/vscode/test/debugger_host/
  function_breakpoints.ts                — NEW: extension-host function breakpoint
  runtime_failure_filters.ts             — NEW: filter configuration and stop behavior
docs/pascal/tools/
  debugger.md                             — MODIFY only after shared behavior exists
  debugger-jsonl.md                       — MODIFY implemented commands only
  debugger-dap.md                         — MODIFY advertised DAP behavior only
```

The exact split may follow existing module boundaries, but do not add new
behavior to the already large `session.rs` or `dap/server.rs` files. Record any
layout deviation in [progress.md](progress.md) before implementing it.

## Work items

### `U20-00` — checkpoint and baseline

Prerequisite: explicit authorization for any required Git mutation.

- Verify the completed `UMB-10D` worktree and parent evidence.
- Establish a recoverable checkpoint, or record that the user explicitly
  authorized continuing in the dirty worktree.
- Capture branch, `git status --short --branch`, focused debugger tests, build,
  VS Code tests, and the known independent workspace baseline.

Exit gate: the new package cannot accidentally mix with or lose `UMB-10D`.

### `U20-01` — freeze contracts and negative tests

Prerequisite: `U20-00` done.

- Encode the decisions from [scope-and-decisions.md](scope-and-decisions.md) in
  tests before new positive behavior.
- Inventory exact DAP capability fields and JSONL schemas.
- Freeze all resource limits and diagnostic-code validation.
- Add negative tests for unknown/ambiguous/no-entry function selectors,
  unknown failure codes, atomic replacement, and protocol parity.

Exit gate: every matrix row has an owning test or an explicit later work item.

### `U20-10` — shared function-breakpoint engine

Prerequisite: `U20-01` done.

- Split the VM breakpoint/session modules before extending them.
- Resolve selectors through `matching_functions` and bind exact entry sequence
  points to one logical session ID.
- Implement bounded, atomic replace/clear/list state.
- Combine source and function IDs deterministically at a sequence boundary.

Exit gate: VM tests cover canonical, short, nested, same-named, no-entry,
recursive, multi-task, collision, limit, and atomicity cases.

### `U20-11` — function-breakpoint adapters

Prerequisite: `U20-10` done.

- Add JSONL commands and schemas over the shared engine.
- Split DAP request handling, advertise function-breakpoint support, and map
  standard `setFunctionBreakpoints` replace-all semantics.
- Add VS Code extension-host coverage using an ordinary source fixture; do not
  add editor-only identity logic.

Exit gate: JSONL, DAP, and VS Code observe the same verification state, match
count, logical ID, and stop behavior.

### `U20-20` — shared runtime-failure policy

Prerequisite: `U20-01` done. May begin only after `U20-11` is checkpointed.

- Expose or consume the central diagnostic-code catalog without duplicating
  runtime codes in adapters.
- Implement validated, bounded, session-local replace-all filter state.
- Preserve default all-stop behavior.
- For a nonmatching failure, emit the diagnostic and terminate with failure
  without creating an inspectable stopped state.

Exit gate: shared tests cover default, exact match, nonmatch termination,
unknown-code atomicity, consecutive failures, and cleanup.

### `U20-21` — runtime-failure adapters

Prerequisite: `U20-20` done.

- Add one JSONL filter command with documented schema and stable errors.
- Advertise DAP exception filters and implement `setExceptionBreakpoints`.
- Verify VS Code can select the advertised filters and receives the same
  stopped/diagnostic/terminated sequence as DAP.

Exit gate: adapter tests prove equivalent state and terminal outcomes; no
adapter reconstructs categories or codes.

### `U20-30` — non-mutating policy completion

Prerequisite: `U20-11` and `U20-21` done.

- Generalize existing condition/hit-count evaluation to function breakpoints.
- Preserve source logpoints and deterministic same-boundary ordering.
- Enforce condition, template, evaluation, output, and binding limits.
- Add rejection tests for unsupported mutating actions and unsupported custom
  DAP fields.

Exit gate: same-boundary logs/stops, hit counters, errors, limits, and
non-mutation pass in shared and adapter tests.

### `U20-40` — current documentation and editor surface

Prerequisite: all accepted behavior above implemented.

- Update only implemented behavior under `docs/pascal/tools/`.
- Add precise examples for canonical/short selectors, multi-match binding,
  failure filter semantics, and termination without a stop.
- Confirm VS Code contribution/configuration text matches advertised DAP
  capabilities.

Exit gate: docs contain no future claims and all examples have test coverage.

### `U20-50` — verification, checkpoint, and closure

Prerequisite: `U20-40` done.

- Run the complete command set in [verification-matrix.md](verification-matrix.md).
- Classify unrelated failures with exact evidence.
- Update parent progress and acceptance rows.
- After an authorized recoverable checkpoint, delete this detail package and
  nominate the next executable umbrella package.

Exit gate: every applicable row is pass or an independently evidenced baseline;
no UMB-20-only unfinished decision remains outside the parent umbrella.
