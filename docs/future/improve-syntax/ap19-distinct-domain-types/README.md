# AP19: Distinct domain types

Status: complete (Q12, Q13; AP19.1, AP19.2). Effort: large.
Completion is tracked in the [central README](../README.md); the process is in
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

### Underlying types and conversions (AP19.1)

- The underlying type of a distinct type is `integer`, `real`, `string`, or
  `boolean`, written directly or through an alias that resolves to one of
  them. Records and enums are rejected because they already have nominal
  identity; arrays, dictionaries, channels, `Option`, `Result`, tasks,
  function types, generic parameters, and other distinct types are rejected.
  Each diagnostic names the allowed underlying types.
- Unwrapping converts only to the exact underlying type; an alias of that
  type is the same type and is accepted. A direct conversion between two
  distinct types is rejected with the hint `OrderId(integer(U))`.
- The built-in type name call form (`integer(X)`, `real(X)`, `string(X)`,
  `boolean(X)`) only unwraps a distinct value. Every other use, such as
  `integer(3.5)`, is an error that names the existing conversion routines.
- `UserId(42)` is a compile-time constant when its argument is a compile-time
  constant; otherwise the conversion is a computed value.
- Distinct values are not unwrapped implicitly for `Std.*` routines, console
  output, or built-in type operations: `Name.Length()` on a
  `distinct string` requires `string(Name).Length()`, and `WriteLn(Id)`
  requires `WriteLn(integer(Id))`.

### Comparisons, constraints, keys, and `case` (AP19.2)

- Equality and ordering apply to two operands of the same distinct type when
  the underlying type supports them. Mixed domains and distinct-versus-underlying
  comparisons are rejected with a conversion hint.
- A distinct type satisfies `Comparable` through its underlying type. It does not
  satisfy `Numeric` (no arithmetic) or `Printable` (no implicit output).
- A distinct type may be a dictionary key.
- A distinct value may be a `case` selector when its underlying type is a valid
  selector type. Value labels and range endpoints are compile-time constants of
  the same distinct type, such as `UserId(1)` or a `const` of type `UserId`; a
  plain `1` is a type mismatch.
- `in` follows equality: `Id in Ids` and `Id in Lookup` accept the same distinct
  type. Substring `in` stays unavailable for a `distinct string`.

## Open decisions

None.

## Dependencies

- AP05 (qualified names for imported domain types).
- AP16 (constant classification, so that `const Admin: UserId := UserId(1);`
  has defined compile-time behavior).

## Order

AP19.1 delivers the type identity and conversions. AP19.2 adds the inherited
comparisons.

## Work packages

- [x] [AP19.1: Distinct type declarations and conversions](01-distinct-declarations.md)
- [x] [AP19.2: Comparisons on distinct types](02-distinct-comparisons.md)

## Acceptance

Domain identities cannot be interchanged accidentally; construction and
unwrapping are explicit, same-type comparisons work, and arithmetic is not
inherited. Representation hiding uses existing record visibility.
