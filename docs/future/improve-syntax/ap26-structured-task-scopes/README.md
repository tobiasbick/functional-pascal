# AP26: Structured task scopes

Status: complete (AP26.1–AP26.3; Q19, Q20, Q24–Q27). Effort: large. Completion is
tracked in the [central README](../README.md); the process is in
[development-process.md](../development-process.md).

AP40 replaces the original detached/standalone-go and anonymous-only ownership
decisions recorded below. Every go is a same-routine scoped expression, ordinary
discard is allowed, and remaining groups join before creating-task completion.
Scope normal/early exit, failure ordering, and handle escapes remain implemented.
See [AP40](../ap40-explicit-task-lifetimes/README.md) and the current handbook;
AP26 remains complete.

Builds on the existing `go` and `task of T` model; it does not introduce
deferred workflows.

Implemented behavior is documented in
[task scopes](../../../pascal/language/concurrency/task-scopes.md).

## Goal

Ownership, detachment, and completion are visible and governed by the same
explained rule on every exit path.

## Decisions

### Completion rule (Q19)

- On normal arrival at `end scope;`, wait for all child tasks to finish without
  requesting cancellation.
- On any other exit from the scope (`return`, `break`, `continue`, `try`
  propagation, or panic), request cancellation of the remaining children and
  then wait for their completion before leaving the scope.
- A failed child also triggers cancellation of the remaining children followed
  by waiting for completion, using the failure rules below.

### Ownership (Q20)

- Every `go` inside a scope starts a child owned by the innermost scope,
  whether used as an expression or a statement. Its handle cannot escape by
  return, storage in an outer variable, or channel send.
- A `go` outside any scope remains detached.
- Keep `Std.Tasks` groups for dynamic cases such as passing groups between
  routines or supervised retries. A scope is not a user-visible task group;
  internal mechanisms may be reused where their contracts fit.
- Reserve `scope` as a keyword.

```pascal
scope
  const A: task of integer := go Fetch(1);
  const B: task of integer := go Fetch(2);
  Show(Wait(A) + Wait(B));
end scope;
```

### Cancellation without tokens (Q24)

- A cancelled scope child stops at its next instruction boundary or wait
  point and ends with the `Cancelled` outcome. It needs no token and no API;
  explicit `CancellationToken` values remain the mechanism of `Std.Tasks`
  groups only.
- Cancelling a child also cancels the children of every scope that child has
  open, because the child leaves those scopes abnormally.

### Routine boundaries (Q25)

- Scope ownership never crosses a routine boundary, named or anonymous. A
  routine called inside a scope does not inherit it.
- A `go` inside an anonymous routine that is lexically inside a scope is a
  compile error unless that anonymous routine opens its own `scope`. The
  diagnostic suggests moving the `go` out of the anonymous routine or opening
  a scope inside it.

### Failures (Q26; revises the failure part of Q20)

- Only a panic or another runtime error of a child fails the scope. A returned
  `Error(...)` is an ordinary value; the parent forwards it explicitly, for
  example with `try Wait(A)`, which is an ordinary abnormal exit (Q19).
- A failed child marks the scope as failed: its siblings **and** the scope body
  are cancelled, the scope waits for every child, and leaving the scope
  re-raises the child's original diagnostic. If the body is waiting on that
  child, `Wait` reports the same failure; it is reported once.
- The primary failure is the first one observed in time, from the body or a
  child. Every further failure is attached to the primary diagnostic in
  child start order. `Cancelled` outcomes are never failures. A failure takes
  precedence over `return`, `break`, `continue`, and `try` propagation.
- A `go` statement inside a scope whose callee returns `result of ...` is a
  compile error: the error would be lost. Keep the handle and forward the
  error with `try Wait(Handle)`.

### Handle escapes (Q27)

- A handle of a scope-owned child may only appear in bindings declared inside
  its scope (including arrays, records, and other values built there), in
  `Std.Tasks` wait operations, and as an argument of a direct routine call,
  including `go` of a sibling, whose parameter is proven not to escape.
- Everything else is a compile error that names the owning scope: return,
  assignment to a binding declared outside the scope (including through `var`
  parameters, fields, and elements), channel send, any capture by an anonymous
  routine, and calls through callable values.
- The compiler infers per parameter whether it can escape (return, outer
  storage, channel send, capture, or forwarding to an escaping parameter). The
  result is recorded in the unit interface, so imported helpers are checked
  without annotations or new syntax.
- There is no runtime fallback: what cannot be proven statically is rejected.

## Open decisions

None for AP26.1, AP26.2, or AP26.3.

## Dependencies

- AP13 (block syntax with named closers).
- AP17 (`var` arguments cannot be passed to `go`).

## Order

AP26.1 provides the scope block with ownership, cancellation, and completion
on every exit. AP26.2 provides failure propagation. AP26.3 enforces handle escape
restrictions.

## Work packages

- [x] [AP26.1: Scope blocks and child completion](01-scope-blocks.md)
- [x] [AP26.2: Child failure propagation](02-failure-propagation.md)
- [x] [AP26.3: Task handle escape restrictions](03-handle-escape-restrictions.md)

## Acceptance

Ownership, detachment, and completion are visible and governed by the same
explained rule on every exit path.
