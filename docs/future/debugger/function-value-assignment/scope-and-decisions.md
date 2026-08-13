# Scope and decisions

## Accepted scope

- FVA-S01: `setVariable`, DAP `setExpression`, JSONL `variable.set`, and JSONL
  `expression.set` may copy a function value from one exact visible source
  binding to an existing mutable function-typed target.
- FVA-S02: the source expression is one visible binding name after parentheses
  are removed by existing expression lowering, for example
  `CurrentHandler := BackupHandler`. The source binding must already contain a
  `Value::Function` and retain portable `DebugType::Function` metadata.
- FVA-S03: the destination may be a mutable initialized or uninitialized root,
  or an already supported stored descendant whose resolved expected type is
  `DebugType::Function`.
- FVA-S04: immutable-capture closures and non-capturing routine values are
  eligible only when the runtime function is not task-bound and its complete
  retained capture graph contains no cells, task handles, opaque handles, or
  nested task-bound function values.
- FVA-S05: local, global, parameter, capture-cell, active payload, record,
  array, and existing dictionary-value destinations retain their current
  ownership, initialization, path, and generation rules.
- FVA-S06: success and failure have identical behavior through the shared VM,
  JSONL, DAP, and VS Code surfaces; no custom protocol command or capability is
  added.

## Fixed decisions

- FVA-D01: assignment copies the existing immutable `SharedFunction`; it does
  not reconstruct the function or deep-clone its environment. This preserves
  the language rule that copying a closure shares its existing environment.
- FVA-D02: both source and destination require portable function type metadata.
  Signature compatibility is structural over parameter order and result type,
  with cycle-safe comparison of referenced portable types. Runtime names,
  arity alone, display strings, and best-effort coercion are insufficient.
- FVA-D03: a `Dynamic` source or destination is rejected even when its current
  runtime value happens to be a function. The first slice must prove the
  declared signature.
- FVA-D04: task-bound function values are rejected for every destination. The
  current mutation target does not retain enough escape or cross-task lifetime
  metadata to prove a narrower write safe.
- FVA-D05: capture eligibility is checked with an explicit bounded graph walk.
  It must not invoke, detach, or otherwise execute the function value.
- FVA-D06: target resolution and source eligibility complete before live state
  changes. A failed lookup, signature check, capture check, storage access, or
  commit preserves the old value and every current inspection handle.
- FVA-D07: a successful assignment uses the existing atomic root commit and
  global invalidation policy. Old frames and variable references expire once;
  clients refresh the stopped snapshot.
- FVA-D08: direct named routines, bound methods, and new closure syntax are not
  fallback source values. The user must first have a visible source binding
  containing the desired materialized function value.
- FVA-D09: explicit inactive-variant transition with a function payload is not
  part of this package. Only an existing resolved function-typed target is
  eligible.

## Stable errors

Reuse existing error families:

- `unknown_name` and `uninitialized_value` for the source binding;
- `variable_not_mutable`, `variable_uninitialized`,
  `variable_path_unsupported`, `variable_target_expired`, and
  `variable_unavailable` for the destination;
- `variable_value_type` for a non-function runtime value, absent function type
  metadata, incompatible signature, task-bound value, forbidden capture graph,
  dynamic endpoint, or unsupported replacement expression shape; and
- existing evaluation-limit and cancellation errors for the bounded operation.

Every `variable_value_type` failure must distinguish signature mismatch from
runtime ownership or capture rejection and include a concrete corrective hint.

## Exit criteria

The slice is complete only when all FVA work packages are complete, every row
in `verification-matrix.md` is `PASS`, current user documentation describes
only verified behavior, and `progress.md` contains exact successful commands.
