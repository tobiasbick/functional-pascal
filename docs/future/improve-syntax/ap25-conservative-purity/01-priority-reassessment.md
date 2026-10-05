# AP25.1: Priority reassessment

Package: [AP25: Conservative purity](README.md)

This is a decision work package. It is complete when the user's decision is
recorded; it changes no code.

## Scope

Decide, after practical use of AP23 contracts, whether AP25 is implemented,
kept at low priority, or closed.

## Prerequisites

- AP23 complete and used in repository code.

## Implementation

- Collect where contracts wanted function calls that only a purity check
  could allow, and where purity would have caught real defects.
- Present the findings and record the user's decision in the package README.

## Affected areas

- `docs/future/improve-syntax/ap25-conservative-purity/`, central README status.

## Migration

None.

## Documentation

Planning documentation only.

## Verification

- The package status records the decision; AP25.2 and AP25.3 are either kept
  with their prerequisites or removed with the package closed.
