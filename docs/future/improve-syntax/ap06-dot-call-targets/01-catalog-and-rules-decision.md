# AP06.1: Catalog, import, and conflict decisions

Package: [AP06: Fixed dot-call targets](README.md)

This is a decision work package. It is complete when the user's decisions are
recorded; it changes no code.

## Scope

Prepare and record the open AP06 decisions: catalog contents, import rules,
name-conflict rules, and the treatment of mutating operations.

## Prerequisites

None. AP05's import model is already decided (Q04).

## Implementation

- Inventory current receiver calls using resolved semantic information, not
  text search. Classify each call by target: record method, standard routine
  on `string`, array, or dictionary, standard routine on another type,
  user-defined free routine, or callable value. Count per category and list
  the standard routines actually used.
- Propose a catalog per receiver type based on the inventory, with each entry
  mapping one name to one standard routine and stating its result type.
- Propose rules for imports, name conflicts, and mutating operations
  (see open decisions 2–4 in the [package README](README.md)).
- Present the proposals to the user and record the decisions.

## Affected areas

- `docs/future/improve-syntax/ap06-dot-call-targets/README.md` (decisions).
- `docs/future/improve-syntax/ap06-dot-call-targets/catalog.md` (new: the
  agreed catalog).
- AP06.2, AP06.3, and AP17.3 files, if the decisions change their scope.

## Migration

None.

## Documentation

Planning documentation only.

## Verification

- Every open decision in the package README is replaced by a recorded decision.
- Every catalog entry names receiver type, dot name, standard routine, and
  result type, and no receiver type has two entries with the same name.
- The inventory counts are recorded in the pull request.
