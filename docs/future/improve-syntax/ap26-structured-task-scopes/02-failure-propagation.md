# AP26.2: Child failure propagation

Package: [AP26: Structured task scopes](README.md)

## Scope

Propagate the first child failure from a scope after cancelling and waiting
for the siblings (Q20).

## Prerequisites

- AP26.1 (scope blocks).

## Implementation

- The first child failure requests sibling cancellation; after all children
  finish, re-raise a panic, or propagate a `Result` error when the enclosing
  routine returns a compatible `Result`; otherwise convert the error to a
  panic.
- Attach further failures to the diagnostic.
- Distinguish cancellation of siblings from their own failures.

## Affected areas

- `crates/fpas-vm/src/vm/tasks/` scope module, compiler scope exit lowering,
  sema checking of `Result` compatibility, VM panic diagnostics.

## Migration

None.

## Documentation

- The task-scope page, `docs/pascal/language/error-handling/panic.md`.

## Verification

- Child panic, child `Result` error with compatible and incompatible
  enclosing routines, multiple failures, failure during abnormal exit of the
  body, sibling cancellation.
