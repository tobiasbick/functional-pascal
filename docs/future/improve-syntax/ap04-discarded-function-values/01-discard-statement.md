# AP04.1: Discard statement

Package: [AP04: Discarded function values](README.md)

## Scope

Add the statement `discard Expression;` and reserve `discard`. Unused function
results remain accepted in this work package.

## Prerequisites

- AP02 (diagnostic codes).

## Implementation

- Lexer: reserve `discard`; diagnose its use as an identifier with a rename hint.
- Parser and AST: the `discard` statement.
- Sema: the operand must produce a value; reject procedure calls; reject task
  handles and values whose type contains task handles according to the rules
  below. Apply the agreed generic, channel, and callable policies consistently.
- Sema and unit interfaces: track and propagate whether callable captures are
  statically proven free of task handles. Preserve this information through
  bindings, assignments, arguments, returns, aggregates, and imported units;
  lost or unavailable information must not be treated as proof.
- Compiler: evaluate the operand exactly once and drop the value.
- Formatter and editor highlighting.

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

Agreed: allow `discard Value;` in generic code only when the declared
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

Agreed: inspect the channel element type recursively, using the same rules as
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

Agreed: allow discard of a function or procedure value only when its stored
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

Capture analysis and propagation across unit interfaces are part of AP04.1.
The current callable signature alone cannot express this proof. Existing
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
- AP04.2 must use the same operand classification when offering `discard` as
  a fix for unused function results, so that its hint never recommends an
  invalid discard.

## Affected areas

- `crates/fpas-lexer/src/token/keywords.rs`, `fpas-parser` statement parsing,
  `fpas-sema/src/check/stmt/`, `fpas-sema/src/check/discard/`,
  `fpas-sema/src/check/closures/`, callable
  propagation and `fpas-unit` interfaces, `fpas-compiler` statement lowering,
  `fpas-fmt`, `editors/vscode/syntaxes/`.

## Migration

Rename any repository identifier spelled `discard`.

## Documentation

- `docs/specs/grammar.ebnf`, keyword list, a section on discarding values in
  the functions or statements documentation.

## Verification

- Tests: ordinary values, `Result`, `Option`, postfix chains, task handles,
  aggregates containing task handles, procedures, and exactly-once evaluation.
- Include aliases, nested aggregates, empty containers, inactive variants,
  recursive types, and record methods returning tasks without stored handles.
- Generic tests: unconstrained and `Printable` parameters rejected, `Numeric`
  and `Comparable` accepted, and the same cases nested in aggregate types.
  Observed call sites must not relax the generic-body check. Concrete task-free
  results of generic calls are accepted; task-containing results are rejected.
- Channel tests: task-free elements accepted; direct and nested task handles
  rejected, including empty channels and channels with additional references.
- Callable tests: capture-free and proven task-free captures accepted; direct,
  nested, and transitive task captures rejected; unknown captures rejected.
  Include captured channels, bound-method receivers, and callable signatures
  accepting or returning tasks without capturing them.
- Verify capture information through bindings, assignments, arguments,
  returns, aggregates, and imported units. Missing information must reject
  discard rather than silently permitting it.
- Include callable default values in omitted record fields.
- Verify that only a direct discarded `go` expression receives the
  `go Worker();` replacement hint.
- Formatter round trip; VS Code grammar verification.

## Result

Static capture proofs are retained through immutable bindings, aggregate
construction (including record defaults), routine results, and unit interfaces.
Mutable storage containing callables is conservatively treated as having
unknown captures, including after assignments. Scalar mutable captures remain
provably task-free when their declared type excludes task handles.

AP04.2 remains responsible for requiring consumption of function results.
