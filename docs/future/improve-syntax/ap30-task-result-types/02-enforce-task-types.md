# AP30.2: Enforce task result types

Package: [AP30](README.md). Status: complete; implemented and verified.
AP30.1 supplies the recorded contract.

## Implemented behavior

Bare `task` is a procedure-task handle in every declaration context. A function
task has type `task of T`; no initializer-specific specialization or result
erasure remains. Parameters, returns, aliases, fields, containers, generic
substitutions, and exported interfaces preserve that distinction.

`Wait` on a procedure task is valid as a statement and cannot supply a value,
including inside another expression or aggregate. Function `Wait` returns the
known T and follows normal result-consumption rules. Homogeneous task arrays
retain their element type. Existing TaskCase/Select observes mixed completion
without consuming or erasing the original handles; scopes retain their escape
and join rules.

Spawned results and calls supplying spawn arguments cannot rely on unbound
callee parameters or inference holes. An enclosing generic parameter remains
valid evidence. A separate evidence model diagnoses unresolved call results
without substituting error wildcards into ordinary parameter or return checks.
Inline callback and routine bodies use their own checking context.

Group and supervised workers have fully instantiated callable signatures with
exactly one by-value CancellationToken parameter. Explicitly typed callable
bindings can specialize generic workers before starting them.

## Implementation owners

- `crates/fpas-sema/src/check/name_resolution/types/resolve.rs` resolves bare
  task to the internal procedure result. `check/decl/vars.rs` preserves the
  declared type without task-specific initializer inference.
- `crates/fpas-sema/src/types/compatibility.rs` and `check/calls/inference.rs`
  enforce the procedure/function distinction, including generic inference,
  and retain error compatibility only for recovery from invalid source.
- `crates/fpas-sema/src/check/expr/tasks.rs`, the shared call-result checker,
  and `check/decl/routine_body.rs` own spawn evidence and separate body contexts.
  `check/expr/calls/mod.rs` rejects procedure Wait in value positions.
- `crates/fpas-sema/src/std_registry/builtins/channel_task/groups.rs` checks
  group-worker signatures and preserves their result types.
- `crates/fpas-compiler/src/lowering/types/expressions.rs`, `lowering/globals.rs`,
  and `lowering/stmt.rs` lower declared task types directly. Task interning
  retains the existing Unit representation; obsolete specialization is removed.
- Compiler and VM debug metadata display procedure handles as `task` and
  function handles as `task of T`. Existing unit interfaces serialize both
  representations without a format change.
- `crates/fpas-language-service/src/intellisense/task_wait.rs` specializes Wait
  signature help to the actual argument, including qualified imports and aliases.
  `lib/api/Std/Tasks.fpas` describes typed function-task families and procedure
  variants through the existing intrinsic declaration exporter.

## Repository consumers and documentation

Library, application, example, FPAS test, and Rust fixture declarations use the
actual function result types. Procedure bindings and arrays remain bare task.
The [task reference](../../../pascal/language/concurrency/task-handles.md),
[spawn reference](../../../pascal/language/concurrency/go.md),
[Std.Tasks](../../../pascal/std/concurrency/task.md), compact reference, grammar
comments, and affected handbook examples describe these current rules.

## Regression coverage

- `crates/fpas-sema/src/tests/stmt/concurrency/task_types.rs` owns source-level
  rejection through bindings, parameters, aliases, records, enums, arrays,
  Option/Result, generics, procedure Wait value positions, unresolved spawn
  results, and invalid group workers. Positive controls cover whole-handle
  generics, nested task results, homogeneous waits, mixed selection, typed
  optional inputs, bound workers, and ordinary inline callbacks.
- Existing compiler concurrency and typed global/local fixtures cover lowering
  and runtime result retrieval, including a function returning a procedure handle.
- `crates/fpas-cli/src/main_tests/projects/exports.rs` covers public task fields,
  arrays, procedure parameters, typed results, and fresh/reused unit interfaces.
  Existing CLI call-shape tests preserve one primary diagnostic for invalid spawns.
- `crates/fpas-language-service/tests/intellisense/task_signatures.rs` covers
  typed function and procedure Wait signatures through an import alias.
- Existing debugger, task-group, supervision, FPAS, and project checks exercise
  migrated consumers and preserved task lifecycle behavior.

## Follow-ups

None within AP30. AP34 owns broader inference policy; AP40 owns additional task
lifetime, shared-state, and wait-priority decisions.
