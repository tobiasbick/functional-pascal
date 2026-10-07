# AP06.1: Catalog names and remaining operation rules

Package: [AP06: Fixed dot-call targets](README.md)

This is a decision work package. It is complete when the user's decisions are
recorded; it changes no code.

## Scope

Validate the [complete operation inventory](catalog.md) and record the
remaining AP06 decisions: canonical names for additional operations,
type-qualified factory forms, and name-conflict rules. Revisit the agreed
implicit-receiver exception with the user as expressly requested. Automatic
availability without the five former Std units and one public call form per
operation are already agreed. Apply the
[agreed standard-operation naming rules](README.md#standard-operation-naming-agreed)
when validating the catalog.

## Prerequisites

None. Built-in type operations do not depend on AP05 imports or aliases.

## Implementation

- Inventory current receiver calls using resolved semantic information, not
  text search. Classify each call by target: record method, standard routine
  on `string`, array, dictionary, `Option`, or `Result`, standard routine on
  another type, user-defined free routine, or callable value. Count per
  category and list the standard routines actually used.
- Inventory ordinary calls, free function references, imports, aliases, and
  generated API declarations for the five former type-helper units as well.
- Validate all 75 existing operations against their implementations and
  current semantics. Preserve distinct operations beyond the base catalog.
  Identify genuine aliases through behavior, constraints, diagnostics, and
  evaluation, rather than names alone. Record every retained operation or
  canonical replacement and the three new `IsEmpty` operations.
- Compare equivalent operations across receiver types: use the agreed
  canonical names, keep corresponding argument roles in the same order, and
  record type-required differences. Do not introduce synonymous dot names
  for the same operation.
- Record one type-qualified factory form for `Chr` and `Fill`, including
  generic array type syntax and inference. Assign `Join` to `array of string`.
  Settle canonical names for equivalent operations across types, including
  string `Substring` and array `Slice`, without losing their behavior.
- Propose rules for name conflicts
  (see the remaining open decisions in the [package README](README.md)).
- The agreed implicit-receiver exception uses ordinary dot calls for
  mutating operations, with writable-receiver checks and explicit `var`
  markers only on written arguments. Discuss this exception again with the
  user before AP06.3/AP17.3, as requested in the
  [package follow-up](README.md#follow-up-discussion), and record the outcome
  in AP06 and AP17. Coordinate the concrete delivery order of AP06.3/AP17.3.
- Present the proposals to the user and record the decisions.

## Affected areas

- `docs/future/improve-syntax/ap06-dot-call-targets/README.md` (decisions).
- `docs/future/improve-syntax/ap06-dot-call-targets/catalog.md` (validate the
  complete operation inventory and recorded names/forms).
- AP06.2, AP06.3, and AP17.3 files, if the decisions change their scope.

## Migration

None.

## Documentation

Planning documentation only.

## Verification

- Every open decision in the package README is replaced by a recorded decision.
- The user-requested follow-up on the implicit-receiver exception has been
  discussed and its outcome recorded in AP06 and AP17 before implementation.
- Every catalog entry names receiver type, canonical dot name, standard
  routine, meaning, argument roles, and result type, and no receiver type has
  two entries with the same name.
- Every existing operation is accounted for as a preserved type operation or
  a verified alias of one canonical replacement. The new `IsEmpty` operations
  are assigned to AP06.3; factory syntax and the receiver exception with its
  follow-up outcome are recorded.
- Automatic availability and removal of the five public units, their ordinary
  calls, and their free function references are reflected in the migration
  plan. Other standard units retain explicit imports.
- A comparison of shared operations confirms the agreed naming rules:
  equivalent meanings share one canonical name, corresponding argument roles
  have comparable order, and type-required differences are documented.
- The inventory counts are recorded in the pull request.
