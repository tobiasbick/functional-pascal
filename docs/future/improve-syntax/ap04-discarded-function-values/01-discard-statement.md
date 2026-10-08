# AP04.1: Discard statement

Package: [AP04: Discarded function values](README.md)

Status: complete.

## Result

`discard Expression;` evaluates a value once and deliberately ignores it.
`discard` is reserved. Procedure calls produce no value and are rejected.

## Operand and type rules

The following rules make the agreed task-handle prohibition explicit:

- The operand must produce a value. Calling a procedure produces no value and
  is rejected; a procedure value follows the callable rules below.
- Reject a task handle directly, including an existing variable or a function
  result of task type.
- Resolve named types and inspect nested array elements, dictionary keys and
  values, stored record fields, every enum variant payload, `Option` payloads,
  and both branches of `Result`. Reject the whole operand if any of these
  types contains a task handle. Traverse recursive types without looping.
- Check the declared type, not the current contents or active variant. An
  empty array of task handles, `None` of an option containing a task handle,
  or a `Result` whose inactive branch contains a task handle is still rejected.
- Record method signatures are not stored fields; a method returning a task
  does not by itself make the record contain a task handle.

For accepted operands, evaluate the expression exactly once. Discarding a
`Result` or `Option` does not unwrap it. No runtime inspection of container
contents is needed for the type check.

### Generic operands

Allow `discard Value;` in generic code only when the declared
constraints prove that the operand type cannot contain task handles for every
permitted type argument. Apply this rule recursively to containers containing
generic parameters; channels and callable captures must also satisfy their
rules below.

- An unconstrained `T` is insufficient, including inside `Option of T`, arrays,
  records, or either branch of `Result`.
- Under the current constraint definitions, `Numeric` and `Comparable` provide
  this guarantee; `Printable` does not exclude task handles.
- Check the generic body against its declared constraints, rather than assuming
  that currently observed call sites cover every possible type argument.
- At a call site with a fully resolved result type, apply the ordinary discard
  rules. A generic function call returning `integer`, for example, may be
  discarded. Calling a generic function does not by itself prevent discard.
- Failure to prove the guarantee is a compile-time error. Use the existing
  constraints, without a runtime check or a new constraint keyword.

### Channels

Inspect the channel element type recursively, using the same rules as
for array elements. Allow discard only when the element type is statically
proven unable to contain task handles.

- `channel of integer` is allowed.
- `channel of task of integer` is rejected.
- Task handles nested inside element types, including `Option`, `Result`,
  records, arrays, and further channels, also prevent discard.
- Callable element types follow the capture rules below; unknown captures do
  not provide proof that the channel cannot contain task handles.
- The rule does not depend on whether the queue is empty or another reference
  to the channel exists at runtime.

### Callable values and closure captures

Allow discard of a function or procedure value only when its stored
captures are statically proven free of task handles.

- Capture-free callables and closures with proven task-free captures are
  allowed.
- Captured task handles prevent discard, including handles nested in captured
  containers, records, channels, or further closures. Inspect captures
  recursively using the same rules as for other operands.
- A bound record method stores its receiver; check the captured `Self` by the
  same rules.
- Unknown captures prevent discard. This includes callable parameters,
  returned callable values, and imported callables when no proof is available.
- Apply the same policy to callable values stored inside containers and record
  fields, without relying on the current contents or active variant.

A callable signature accepting or returning a task does not itself establish
that the callable stores a live task handle and is not a reason to reject
discard: discarding a callable value does not invoke it. Conversely, a closure
can capture a task even when its signature mentions only ordinary values.

Capture analysis and unit-interface metadata provide this proof; a callable
signature alone cannot express it. Existing
task-bound closure checks describe mutable state and task affinity; they are
not proof that a closure does or does not capture task handles. Keep both
properties separate, and perform the discard check at compile time.

Keep these discard rules separate from the scope provenance and escape rules
planned in [AP26.3](../ap26-structured-task-scopes/03-handle-escape-restrictions.md).

## Diagnostics

- For `discard go Worker();`, suggest the existing statement `go Worker();`.
- For an existing task handle, a function returning a task, or an aggregate
  containing task handles, explain which type or nested field prevents discard
  and advise retaining and consuming the handles. Do not suggest spawning a
  new task as a replacement.
- Unused-result diagnostics use the same operand classification when offering `discard` as
  a fix for unused function results, so that its hint never recommends an
  invalid discard.

## Implementation

Checking lives in `crates/fpas-sema/src/check/discard/` and callable capture
analysis. Immutable bindings, aggregate construction/defaults, results and
unit interfaces carry capture proofs. Mutable storage containing callables is
conservatively unknown; scalar mutable captures can remain provably task-free.

## Regression coverage

Tests cover recursive types, generics, channels, unknown and transitive captures,
bound methods, defaults, imported proofs, task rejection and exactly-once
execution. See [discard](../../../pascal/language/functions/discard.md).
