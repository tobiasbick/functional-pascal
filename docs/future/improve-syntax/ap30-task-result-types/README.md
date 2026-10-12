# AP30: Sound task result types

Status: Decision pending. Effort: medium. The reproduced type-checking defect
is confirmed; the complete source-level task type contract remains open.
Completion is tracked in the [central plan](../README.md).

## Finding

The [task reference](../../../pascal/language/concurrency/task-handles.md)
requires `task of T` where no initializer determines T. Currently a parameter
`Job: task` permits `return Wait(Job)` in a string-returning function even
when its caller supplies an integer task. Checking succeeds; using the result
as a string fails at runtime. Bare task resolves to an internal error type
that compatibility treats as a wildcard.

## Proposed direction and open decisions

- Never let an unresolved task result pass successful semantic analysis.
- Decide whether bare `task` remains initializer-only inference or disappears.
- Define a usable type for procedure tasks; the internal unit result is not
  currently an ordinary source-language type.
- Keep heterogeneous completion observation separate from typed result
  retrieval. Decide how erased handles may be represented without permitting
  `Wait` to manufacture an arbitrary result type.

## Dependencies and order

AP26 supplies existing scope rules. AP30.1 precedes AP30.2. AP40 can then build
on sound handles without owning the result-typing repair.

## Work packages

- [ ] [AP30.1: Task type contract](01-type-contract.md)
- [ ] [AP30.2: Enforce task result types](02-enforce-task-types.md)

## Acceptance

No checked program can receive an integer through a string-typed task result.
Function tasks, procedure tasks, aggregates, generic calls, unit interfaces,
and non-consuming waits have explicit, tested contracts. Planning does not
approve any particular new spelling.
