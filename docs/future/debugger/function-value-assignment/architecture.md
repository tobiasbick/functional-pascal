# Architecture

## Existing foundations

- `fpas_bytecode::Value::Function` stores an immutable `SharedFunction` with a
  `FunctionId`, canonical name, capture values, and `task_bound` flag.
- Portable `DebugType::Function` already records ordered parameter types and a
  result type. Source-visible debugger bindings retain a `DebugTypeId`.
- `InspectionSnapshot` retains each visible binding's runtime value and
  portable type. Typed source lookup returns the runtime value plus
  `DebugTypeId` without changing ordinary expression resolution.
- The expression parser already lowers a visible binding reference to
  `DebugExpression::Name`. Bare qualified callable names are invocation-only,
  and closure construction is deliberately rejected.
- Mutation targets already retain their expected `DebugTypeId`, generation,
  ownership root, initialization state, and resolved aggregate path.
- `mutation::validate` accepts an eligible function replacement only at the
  mutation root; `mutation::commit` atomically replaces the completed runtime
  value without function-specific storage logic.

The implemented boundary is therefore not a new protocol or a new runtime value.
It is typed source provenance plus bounded function signature and capture
validation before the existing commit.

## Prepare flow

```text
protocol replacement source
  -> existing parser and DebugExpression
  -> resolve destination before replacement evaluation
  -> if destination is DebugType::Function:
       require DebugExpression::Name
       resolve visible source binding as (Value, DebugTypeId)
       prove source and destination function signatures compatible
       prove Value::Function, task_bound = false, safe bounded captures
       clone the existing SharedFunction value
  -> existing atomic mutation commit
  -> existing stopped-snapshot invalidation and protocol result
```

Non-function destinations continue through the current general expression
evaluator and validator unchanged. Function assignment must not make bare
callable names evaluate as values and must not relax controlled-call policy.

## Ownership boundaries

```text
crates/fpas-vm/src/vm/debug/inspection/
  typed_bindings.rs
    — exact lexical source binding lookup returning value plus DebugTypeId

crates/fpas-vm/src/vm/debug/mutation/function_value/
  mod.rs
    — prepare one eligible SharedFunction replacement
  signature.rs
    — cycle-safe structural portable-type compatibility
  captures.rs
    — bounded non-executing retained capture graph validation

crates/fpas-vm/src/vm/debug/session/mutation.rs
  — select function preparation after target resolution; reuse commit
```

`evaluation/execute.rs` is already 473 lines and must not absorb function
assignment. `mutation/validate.rs` is the generic value validator and should
delegate the function-specific concern instead of growing a second capture
policy inline.

## State and lifetime model

- The source and target belong to the same stopped session snapshot. Normal
  frame selection and lexical shadowing choose the source binding.
- Cloning `Value::Function` increments the existing shared immutable function
  storage. The old destination is released only after a successful commit.
- A source function with mutable cells or other live handles can extend or
  export runtime identity. The first slice rejects that entire graph rather
  than guessing destination lifetime.
- Destination writes retain current root semantics: frame registers belong to
  the selected task, globals use the existing shared global store, and closure
  capture targets retain their existing cell identity.
- Failure before or during commit leaves both function values and inspection
  generations unchanged.

## Metadata and validation invariants

- Source and destination `DebugTypeId` values must resolve inside the same
  verified executable.
- Both top-level types must be `DebugType::Function`.
- Structural comparison follows parameter order and result type recursively;
  record and enum identity remains executable-layout identity.
- The graph walk counts every visited runtime value against an explicit limit,
  preserves shared-node detection, and rejects cells, tasks, opaque handles,
  and task-bound nested functions at any depth.
- No display name, runtime `FunctionId`, capture count, or rendered type string
  substitutes for portable type proof.
