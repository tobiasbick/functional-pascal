# Consciously deferred

The following capabilities remain outside the bounded function-value assignment
slice.

- Constructing a new closure from `function` or `procedure` expression syntax
  in debugger evaluation.
- Materializing a named routine, nested routine, static routine, or bound record
  method as a new first-class value when no visible source binding already
  contains it.
- Accepting calls, properties, methods, constructors, record updates, or other
  computed expressions whose result happens to be a function value.
- Copying a function value from a record field, array element, dictionary value,
  payload child, evaluation-result handle, or other non-root source expression.
- Assigning task-bound closures or any function whose retained graph includes a
  mutable cell, task handle, opaque handle, or nested task-bound function.
- Proving same-task-only escape constraints for task-bound values or adding
  destination lifetime and escape metadata solely for debugger assignment.
- Assigning function values to `Dynamic` storage or inferring a signature from
  a runtime function name, `FunctionId`, arity, or display text.
- Constructing an inactive enum, `Result`, or `Option` variant whose payload is
  a function value; the initial slice requires an existing function-typed
  destination path.
- Editing a function's captures, target `FunctionId`, task-bound flag, code,
  source mapping, or callable signature in place.
- Reviving stale frames, variable references, source values, or mutation
  targets.
- Assigning task handles, capture cells themselves, or opaque hosted resources.
- Forcing return values, changing the instruction pointer, restarting frames,
  or adding breakpoint actions that mutate callable state.

Each later package must define its own stable IDs, portable type proof,
identity and lifetime rules, atomicity boundary, protocol parity, privacy and
resource limits, negative coverage, and resume log.
