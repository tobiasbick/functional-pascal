# AP26.2: Child failure propagation

Package: [AP26: Structured task scopes](README.md)

## Scope

A child panic or runtime error fails its scope: siblings and body are
cancelled, all children are awaited, and leaving the scope re-raises the
primary failure with further failures attached (Q26). Returned `Error(...)`
values remain ordinary values.

## Prerequisites

- AP26.1 (scope blocks).

## Implementation

- VM: the first child failure marks the scope failed and cancels its children
  and the scope body at their next instruction boundary or wait point. After
  all children finish, re-raise the primary failure (first observed, body or
  child) and attach further failures in child start order. Cancelled outcomes
  are not failures; a failure observed through `Wait` on the failed child is
  reported once. Scope-owned failures never abort the whole program directly.
- Failure observation identities, retained by task waits, distinguish a
  repeated observation from independent failures with equal diagnostics.
  Synchronous callbacks observe their caller's cancellation controls without
  inheriting its spawn ownership. Blocking main-task waits probe cancellation
  even when an unrelated detached task remains pending.
- Sema: reject a `go` statement inside a scope whose callee returns
  `result of ...`, with a hint to keep the handle and use `try Wait(Handle)`.

## Affected areas

- `crates/fpas-vm/src/vm/tasks/` scopes module, scheduler failure ownership,
  main-thread blocking waits, debugger task driver; sema `go` checks.

## Migration

None.

## Documentation

- The task-scope page, `docs/pascal/language/error-handling/panic.md`.

## Verification

- Child panic with sibling and body cancellation; returned `Error` handled
  through `try Wait`; rejected `go` statement of a `Result` function; multiple
  failures; failure during an abnormal exit of the body; failure observed by
  `Wait` on the failed child; nested scopes.

Regression ownership: compiler concurrency tests cover scope failure and
ordinary returned errors. VM scope-failure tests cover primary selection,
further-failure ordering, and causal deduplication. VM callback and blocking
wait tests cover cancellation during synchronous execution; debugger scope
tests cover joining before the original failure is reported.
