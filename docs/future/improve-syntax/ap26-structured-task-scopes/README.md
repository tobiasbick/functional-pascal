# AP26: Structured task scopes

Status: agreed direction (Q19, Q20). Effort: large. Completion is tracked in
the [central README](../README.md); the process is in
[development-process.md](../development-process.md).

Builds on the existing `go` and `task of T` model; it does not introduce
deferred workflows.

## Goal

Ownership, detachment, and completion are visible and governed by the same
explained rule on every exit path.

## Decisions

### Completion rule (Q19)

- On normal arrival at `end scope;`, wait for all child tasks to finish without
  requesting cancellation.
- On any other exit from the scope (`return`, `break`, error propagation, or
  panic), request cancellation of the remaining children and then wait for
  their completion before leaving the scope.
- A failed child also triggers cancellation of the remaining children followed
  by waiting for completion, using the failure propagation rules below.

### Ownership and failures (Q20)

- Every `go` inside a scope starts a child owned by the innermost scope,
  whether used as an expression or a statement. Its handle cannot escape by
  return, storage in an outer variable, or channel send.
- A `go` outside any scope remains detached.
- The first child failure requests cancellation of its siblings. After all
  children finish, the scope ends with that failure: re-raise a panic, or
  propagate a `Result` error if the enclosing routine returns a compatible
  `Result`. Otherwise convert the error to a panic. Attach further failures
  to the diagnostic.
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

## Open decisions

Before AP26.3, specify the capture limits for closures holding scope-owned task
handles, including the proof needed when passing them to imported helpers.
AP26.3 requires this decision and must preserve the agreed no-escape rule.

## Dependencies

- AP13 (block syntax with named closers).
- AP17 (`var` arguments cannot be passed to `go`).

## Order

AP26.1 delivers the scope block with ownership and completion on every exit.
AP26.2 adds failure propagation. AP26.3 enforces handle escape restrictions.

## Work packages

- [ ] [AP26.1: Scope blocks and child completion](01-scope-blocks.md)
- [ ] [AP26.2: Child failure propagation](02-failure-propagation.md)
- [ ] [AP26.3: Task handle escape restrictions](03-handle-escape-restrictions.md)

## Acceptance

Ownership, detachment, and completion are visible and governed by the same
explained rule on every exit path.
