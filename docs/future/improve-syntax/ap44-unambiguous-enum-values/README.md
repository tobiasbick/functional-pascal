# AP44: Unambiguous enum values

Status: agreed direction; implementation pending.
Effort: small–medium. Completion is tracked in the [central plan](../README.md).

## Problem

Payload-free enum variants may currently share an integer backing value.
Equality and runtime matching then identify them as the same value, while case
coverage counts their variant names separately. A case selecting the second
variant can execute the first variant's arm while requiring both arms statically.
The current [enum specification](../../../pascal/language/types/enums.md) even
allows equal backing values for stable sorting.

An enum may also mix a fieldless variant with an explicit backing value and a
different variant with payload fields. The semantic checker discards all backing
values when any variant has a payload. Accepted syntax must not silently lose
its declared meaning.

## Planned contract

- All backing values within a payload-free enum must be unique, including
  collisions between explicit values and automatically assigned successors.
- If any variant has payload fields, no variant in that enum may declare an
  explicit integer backing value. Fieldless alternatives remain allowed.
- Preserve unique explicit values, existing implicit numbering and overflow
  checks, nominal identity, type-qualified variants, and existing case rules.
- Do not add enum aliases, alternate matching modes, or new conversion syntax.

This replaces AP42's permission for equal backing values in distinct variants;
stable sorting itself is retained. It also tightens AP24's mixed-variant
declarations. Those packages retain their completed status and other contracts.
The restrictions are planned behavior, not the current language specification.

## Dependencies and work packages

AP24, AP39, AP41, and AP42 are complete and provide the relevant enum and case
contracts. No dependency on AP45 or AP46 is required.

- [ ] [AP44.1: Enforce unambiguous enum values](01-enum-value-checks.md)

## Open decisions

None within this bounded contract. Planning does not mark implementation complete.

## Acceptance

Every accepted enum variant has an unambiguous identity for equality and case
coverage. Ambiguous or ineffective backing values fail at the declaration with
an actionable diagnostic. Consumers, cached unit interfaces, tooling, and
current documentation agree with the restriction before completion.
