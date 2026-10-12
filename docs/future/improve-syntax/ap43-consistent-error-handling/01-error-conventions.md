# AP43.1: Error conventions

Package: [AP43](README.md). Status: Decision pending; decision work package.

## Scope and deliverable

Record the user's decision on parser names, Result returns, removal or
retention of aborting wrappers, and the role of Option/Result helper methods.
Distinguish canonical authoring guidance from deleting valid APIs. Inventory
recoverable parsing versus deliberate Unwrap/panic and expected absence.
Describe the existing try matrix independently from proposed API changes.

## Prerequisites and affected areas

AP04/AP20/AP21 are the baseline. Consult AP33.1 for text semantics and AP35.1/
AP39.1 before choosing examples that require their proposed contexts. Change
planning files only; do not introduce another propagation syntax.

## Documentation and verification

The decision must cover Result<T,E> propagation into Result<U,E> with compatible
errors, Option<T> into Option<U>, invalid cross-family propagation, and panic
not being caught. Show eager UnwrapOr and lazy fallback as different operations
with observable effects. Retain useful combinators unless a specific redundant
surface and its migration have been agreed.
