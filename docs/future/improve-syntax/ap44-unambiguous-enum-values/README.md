# AP44: Unambiguous enum values

Status: complete (AP44.1).
Effort: small–medium. Completion is tracked in the [central plan](../README.md).

## Design boundary

Duplicate integer backing values would give different variant names the same
runtime identity while case coverage counts those names separately. AP44 rejects
that ambiguity at the declaration. The
[enum specification](../../../pascal/language/types/enums.md) describes the
implemented rule.

Data enums use variant tags. Backing annotations have no meaning in that
representation and are rejected throughout the declaration.

## Implemented contract

- All backing values within a payload-free enum must be unique, including
  collisions between explicit values and automatically assigned successors.
- If any variant has payload fields, no variant in that enum may declare an
  explicit integer backing value. Fieldless alternatives remain allowed.
- Preserve unique explicit values, existing implicit numbering and overflow
  checks, nominal identity, type-qualified variants, and existing case rules.
- No variant-alias syntax, alternate matching mode, or new conversion syntax
  is introduced. Existing type aliases retain their enum identity and values.

This replaces AP42's permission for equal backing values in distinct variants;
stable sorting itself is retained. It also tightens AP24's mixed-variant
declarations. Those packages retain their completed status and other contracts.
Current language documentation, diagnostics, editor analysis and compiled-unit
checking enforce the same restrictions.

## Dependencies and work packages

AP24, AP39, AP41, and AP42 are complete and provide the relevant enum and case
contracts. No dependency on AP45 or AP46 is required.

- [x] [AP44.1: Enforce unambiguous enum values](01-enum-value-checks.md)

## Open decisions

No selected AP44 decisions or implementation work remain open.

## Acceptance

Every accepted enum variant has an unambiguous identity for equality and case
coverage. Ambiguous or ineffective backing values fail at the declaration with
FP3044 or FP3045 and an actionable repair. Cached interfaces preserve valid
values; incompatible compiler identities force declaration rechecking without
changing the artifact format. Independent subagent review found no blocking
issues. [AP44.1](01-enum-value-checks.md) records ownership, regression coverage
and independent baseline verification exceptions.
