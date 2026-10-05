# AP12.1: Callable expression decision

Package: [AP12: Callable expressions](README.md)

This is a decision work package. It is complete when the user's decision is
recorded; it changes no code.

## Scope

Decide whether AP12 proceeds, and fix its evaluation order and the set of
call targets.

## Prerequisites

- AP06.1 (dot-call decisions), so the relation between callable fields and
  methods is settled.

## Implementation

- Inventory which call targets the current parser and checker accept.
- Present the proposal and the evaluation order to the user; record the
  decision in the package README.
- If rejected, mark AP12 as rejected in the central README and remove AP12.2.

## Affected areas

- `docs/future/improve-syntax/ap12-callable-expressions/`.
- `docs/future/improve-syntax/README.md` (status).

## Migration

None.

## Documentation

Planning documentation only.

## Verification

- The package status changes from proposal to agreed direction or rejected.
