# AP41.2: Qualified-name rules

Package: [AP41](README.md). Status: blocked by AP41.1.

## Scope and affected areas

Implement only the selected enforcement level. Inspect
`crates/fpas-sema/src/check/name_resolution/`, import and enum registration,
formatter handling, and language-service auto-import/completion/rename.
If the decision is authoring policy only, do not change accepted syntax.

## Migration and documentation

Qualify repository uses by their resolved target. Preserve alias hiding,
case-insensitive identity, and local declaration shadowing. Update
[units](../../../pascal/program-structure/units.md),
[enums](../../../pascal/language/types/enums.md), grammar when affected,
compact reference, templates, snippets, and authoring skills.

## Verification

Add colliding imports/variants and confirm existing qualified code resolves
unchanged. Test source and compiled units, patterns, generic variants,
renames, auto-imports, and useful errors or lint messages for the alternative
forms. Run checks appropriate to the selected tooling or language scope.
