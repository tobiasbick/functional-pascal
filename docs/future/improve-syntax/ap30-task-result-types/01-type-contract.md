# AP30.1: Task type contract

Package: [AP30](README.md). Status: complete; decision work package.

## Scope

Fixed task types prevent result erasure while preserving procedure tasks and
completion observation. AP26 supplies existing scope ownership and join rules;
no deferred feature is required.

## Recorded decision

The user delegated selection of the LLM-friendly model. The
[package decisions](README.md#decisions) are the authoritative rule:
`task` is a procedure-task handle, `task of T` is a function-task handle,
and neither is an erased type. Wait retrieves a typed function result or
finishes a procedure task without a value. Homogeneous task arrays retain
their element type; existing TaskCase/Select values serve mixed completion
selection. Existing scopes join mixed child tasks.

No new keyword or public unit/void type is needed. No inference expansion,
new observer API, implicit task conversion, or broad ownership change is
part of the decision.

## Annotation rules

- Function-task bindings use `Job: task of ReturnType := go FunctionName(...)`
  with the resolved function result type, including group and supervised starts.
- Procedure-task bindings use `Job: task := go ProcedureName(...)`.
- Parameters, fields, aliases, and containers carrying function results name
  their actual T; a later use of Wait cannot determine or change it.
- Homogeneous function-task lists use `array of task of T`; procedure lists
  use `array of task`. Mixed lists cannot rely on a wildcard to check.
- Generic signatures preserve the true result type. Std wait signatures
  describe the typed family and procedure-task case without erasing results.

## Implementation ownership and documentation

[AP30.2](02-enforce-task-types.md) records the compiler, standard-library,
tooling, and regression owners. The current
[task handbook](../../../pascal/language/concurrency/task-handles.md) and
[grammar](../../../specs/grammar.ebnf) describe the fixed procedure/function
handle distinction selected by this contract.

## Verification

The source rules, allowed/rejected contexts, annotations, and mixed completion
path are recorded. All AP30 design questions are resolved;
[AP30.2](02-enforce-task-types.md) owns implementation and regression coverage.
