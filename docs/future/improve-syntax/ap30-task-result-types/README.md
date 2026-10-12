# AP30: Sound task result types

Status: Complete; all work packages implemented and verified. Effort: medium.
AP30.1 supplies the selected type contract; AP30.2 owns its implementation.
Completion is tracked in the [central plan](../README.md).

## Scope

The [task reference](../../../pascal/language/concurrency/task-handles.md)
defines procedure and function handles with fixed types. Task result typing
prevents integer values from passing through string-typed Wait results,
including through generics, aggregates, and independently compiled units.

## Decisions

- Bare `task` means a retained procedure task with no result. Its meaning is
  fixed in every annotation context; it never infers a function result and is
  never an erased handle or wildcard.
- A retained function task has type `task of T`, where T is the exact return
  type of the spawned function. Explicit annotations use that form for local
  bindings, parameters, fields, containers, and exported signatures alike.
  AP22's existing annotation rules remain unchanged.
- A procedure spawn produces `task`; a function spawn produces `task of T`.
  Assignment between those categories is rejected. Result types retain their
  ordinary identity/compatibility; task typing introduces no implicit result
  conversion. The internal unit result needs no new public value type.
- Generic `task of T` parameters and aggregate fields accept function tasks
  only; inference cannot bind T to the internal procedure result. A generic
  over an entire handle may infer T as `task`. A function returning such a
  handle produces `task of task`, preserving both levels of the type.
- `Wait` on `task of T` returns T and consumes the result under the existing
  single-consumption rule. `Wait` on `task` is a procedure operation with no
  value: `Wait(Job);` is valid, but assigning its result is rejected.
- WaitAll, WaitAny, and controlled completion waits accept homogeneous arrays
  of procedure tasks or homogeneous arrays of `task of T`. They observe
  completion without consuming results. They do not erase element types.
- Mixed-result completion selection uses the existing `TaskCase`/`Select`
  mechanism and its common `WaitCase` values. A case observes completion only;
  it cannot recover a value of an arbitrary type. Keep the original typed
  handles for `Wait`. Preserve scope ownership and existing case lifetimes.
- For completion of all differently typed children, use the existing task
  scope; outside a shared scope, wait on the separately typed handles. Do not
  introduce heterogeneous task arrays, implicit erasure, another observer type,
  or an additional wait API in AP30.
- Never let an unresolved task result pass successful semantic analysis. The
  compiler's error-recovery type is not a successful task result type.
- Generic call results used by a spawn must be known from arguments or an
  explicitly bound callable type. Ordinary parameter and return checks retain
  their real types; inline callback bodies use their own routine context.
- Group and supervised workers have fully instantiated callable signatures
  with exactly one by-value CancellationToken parameter. A generic routine
  can be bound to the explicit worker type before starting it.

## Examples

These fragments use the implemented rules. The containing program imports
Std.Tasks and declares the routines.

```pascal
const NumberJob: task of integer := go ComputeNumber();
const WorkerJob: task := go RunWorker();
const Number: integer := Wait(NumberJob);
Wait(WorkerJob);
```

A field or parameter uses exactly the same annotation. A function returning
a handle uses `task of T`; a function returning a procedure-task handle uses
`task`. `array of task` contains only procedure-task handles, while
`array of task of integer` contains only integer-result handles. None/Option,
Result, generic substitutions, and aliases preserve the selected handle type.

## Rationale

The distinction mirrors Pascal's function/procedure distinction without a new
keyword. Each annotation has one meaning independent of its initializer, and
there is one explicit annotation for a given handle type. Mixed event
selection already has a safe common representation; reuse it instead of
turning result-bearing handles into untyped values. The cost is writing the
return type on function-task bindings and using explicit selection for mixed
results. No result can be inferred from its receiving variable after erasure.

## Open decisions

None for AP30's selected scope. AP40 continues to own broader task lifetime,
shared-state, and wait-priority decisions.

## Implementation and dependencies

[AP30.2](02-enforce-task-types.md) records implementation owners, migrated
consumers, documentation, and regression coverage. AP26 supplies existing scope
rules. AP40 can build on sound handles without owning the result-typing repair.

## Work packages

- [x] [AP30.1: Task type contract](01-type-contract.md)
- [x] [AP30.2: Enforce task result types](02-enforce-task-types.md)

## Acceptance

An integer-result handle cannot be stored or passed as a string-result handle.
Function tasks, procedure tasks, aggregates, generic calls, unit interfaces,
and non-consuming waits follow the selected contract with regression coverage.
The compiler, handbook, tooling, and repository consumers follow this contract.
