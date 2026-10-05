# AP01.2: Plan spelling review

Package: [AP01: Pascal conventions](README.md)

## Scope

Check every package of this plan against the reference style and the AP01
decisions, and correct inconsistent draft spellings in the planning documents.

## Prerequisites

- AP01.1 (reference examples).

## Implementation

- Review all package and work package files for type applications, block
  endings, binding keywords, parameter forms, and call forms.
- Replace any angle-bracket type application in draft examples with the `of`
  form, except the routine type-parameter declaration retained by AP24.
- Where a draft uses a non-Pascal spelling for a concept that Pascal or Delphi
  already names, either change it or reference the recorded decision that
  allows it.

## Affected areas

- Planning documents under `docs/future/improve-syntax/`.

## Migration

None.

## Documentation

Planning documentation only.

## Verification

- A search for angle brackets in positive draft code finds only routine
  type-parameter declarations, apart from comparison operators. Deliberately
  rejected spellings and current/legacy migration input remain labelled.
- Every remaining exception is linked to a recorded decision.

## Result

[Spelling review](spelling-review.md) covers all package and work package
documents, records the corrections, and links the existing decision gates.
Delivery is local; the completion checkbox awaits merge.
