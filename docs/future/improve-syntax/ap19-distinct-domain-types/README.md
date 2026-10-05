# AP19: Distinct domain types

Status: agreed direction (Q12, Q13). Effort: large. Completion is tracked in
the [central README](../README.md); the process is in
[development-process.md](../development-process.md).

## Goal

Domain identities cannot be interchanged accidentally; construction and
unwrapping are explicit, same-type comparisons work, and arithmetic is not
inherited. Representation hiding uses existing record visibility.

## Decisions

### Spelling (Q12)

- Use `type UserId = distinct integer;` for a distinct domain type.
- Introduce `distinct` as a keyword. This is an explicitly agreed exception to
  the Pascal/Delphi spelling preference: it makes the distinction from the
  alias `type UserId = integer;` visible without a repeated `type` keyword.
  The unselected `type UserId = type integer;` form is diagnosed with the
  canonical spelling.

### Operations (Q13)

- Construction and unwrapping require explicit conversions: `UserId(42)` and
  `integer(Id)`.
- Equality and ordering are inherited from the underlying type for operands
  of the same distinct type. Different domain types are not interchangeable.
- Arithmetic is not inherited; `Id + 1` is invalid. Use records or explicit
  functions for quantities that need arithmetic.
- Do not add a separate declaration form for hiding the representation.
  Records with non-public fields provide that encapsulation.
- Distinctness alone does not validate a URL, path, or duration.

## Open decisions

- Whether a distinct type satisfies generic constraints such as `Comparable`
  through its underlying type, and whether it can be a dictionary key or a
  `case` label. Decide before AP19.2.

## Dependencies

- AP05 (qualified names for imported domain types).
- AP16 (constant classification, so that `const Admin: UserId := UserId(1);`
  has defined compile-time behavior).

## Order

AP19.1 delivers the type identity and conversions. AP19.2 adds the inherited
comparisons.

## Work packages

- [ ] [AP19.1: Distinct type declarations and conversions](01-distinct-declarations.md)
- [ ] [AP19.2: Comparisons on distinct types](02-distinct-comparisons.md)

## Acceptance

Domain identities cannot be interchanged accidentally; construction and
unwrapping are explicit, same-type comparisons work, and arithmetic is not
inherited. Representation hiding uses existing record visibility.
