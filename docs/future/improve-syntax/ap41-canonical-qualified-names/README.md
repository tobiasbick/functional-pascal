# AP41: Canonical qualified names

Status: Decision pending. Effort: medium. Completion is tracked in the
[central plan](../README.md).

## Findings

[Enum variants](../../../pascal/language/types/enums.md) may use short or
type-qualified names. Adding another enum can invalidate earlier short-name
uses; a same-named type can hide the short variant. Plain
[imports](../../../pascal/program-structure/units.md) allow short and full
names simultaneously, while an alias permits only the alias-qualified form.

## Proposed direction and open decisions

Prefer `Type.Variant` for user enums and a single explicitly qualified import
style. Choose whether this is initially authoring/lint policy or mandatory
language syntax; a canonical style alone does not remove alternative legal
forms. If mandatory, select alias requirements versus full qualification.
Built-in Some/None/Ok/Error are reserved forms and need an explicit retained
distinction or separate decision, not accidental renaming.

## Dependencies and work packages

AP05/AP20/AP24 are complete. AP41.1 precedes AP41.2. Retain completed import
tooling; do not reintroduce free-function receiver lookup from before AP06.

- [ ] [AP41.1: Qualification decision](01-qualification-decision.md)
- [ ] [AP41.2: Qualified-name rules](02-qualified-name-rules.md)

## Acceptance

Canonical generated code exposes each imported symbol's origin and each
user enum variant's type. Adding a colliding import or variant cannot change
already qualified targets. Language enforcement matches the recorded scope.
