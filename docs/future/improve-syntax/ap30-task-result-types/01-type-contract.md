# AP30.1: Task type contract

Package: [AP30](README.md). Status: Decision pending; decision work package.

## Scope and prerequisites

Reproduce the mismatch described in the package README against the current
checkout. Inspect existing `task`, `task of T`, procedure spawns, `WaitAll`,
`WaitAny`, task groups, and exported signatures before choosing a rule.
AP26 is complete; no deferred feature is needed.

## Decision deliverable

Record the user's choice for bare annotations, procedure task spelling, and
heterogeneous observation. Distinguish unknown result inference from a real
erased type and from the compiler's error-recovery type. Recommend permitting
result retrieval only when its type is known. Include migration examples for
parameters, fields, arrays, and procedure tasks; do not silently make existing
completion APIs unusable.

## Affected areas and documentation

Planning files in this directory only. Inspect
`crates/fpas-sema/src/check/name_resolution/types/resolve.rs`,
`crates/fpas-sema/src/types/compatibility.rs`, and
`crates/fpas-sema/src/std_registry/builtins/channel_task.rs`.
The current [task handbook](../../../pascal/language/concurrency/task-handles.md)
and [grammar](../../../specs/grammar.ebnf) remain unchanged at this step.

## Verification

The recorded decision covers all listed contexts, specifies allowed and
rejected examples, and resolves every AP30 open decision before AP30.2 starts.
