# AP41.1: Qualification decision

Package: [AP41](README.md). Status: Decision pending; decision work package.

## Scope and deliverable

Record the user's choice of style/lint versus mandatory qualification for
imports and user enums. Compare alias-only calls with full unit-qualified
calls, considering long namespaces, local shadowing, exported types, and
editor auto-import. Specify built-in variant treatment. AP05/AP20/AP24 are
the current baseline, not evidence that their optional forms were mistakes.

## Affected areas and documentation

Planning files only. Inspect name resolution and existing qualified-import
tooling before selecting restrictions. Current
[units](../../../pascal/program-structure/units.md) and
[enums](../../../pascal/language/types/enums.md) stay unchanged.

## Verification

The decision states one generated-code convention, whether other forms remain
legal, and accepted/rejected outcomes for colliding units, later enum
declarations, type/variant collisions, and aliases. Migration must resolve
symbols semantically rather than blindly prefix matching identifiers.
