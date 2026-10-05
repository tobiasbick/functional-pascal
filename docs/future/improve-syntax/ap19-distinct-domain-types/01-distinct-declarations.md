# AP19.1: Distinct type declarations and conversions

Package: [AP19: Distinct domain types](README.md)

## Scope

Add `type UserId = distinct integer;` with a separate type identity and
explicit construction and unwrapping conversions. No operators are inherited
in this work package.

## Prerequisites

- AP16.1 (constant classification for conversions in `const` declarations).
- AP05.1 (recorded package dependency; imported domain types are tested with
  plain and aliased imports).

## Implementation

- Lexer: reserve `distinct`; diagnose its use as an identifier.
- Parser: `distinct` type bodies; diagnose `type X = type Y` with the
  canonical spelling.
- Sema: nominal identity separate from aliases; `UserId(42)` and
  `integer(Id)` conversions; implicit conversions in either direction are
  errors; arithmetic is rejected with a hint to use a record or function.
- Unit interfaces carry distinct identities across compiled units.

## Affected areas

- `crates/fpas-lexer/src/token/keywords.rs`, `fpas-parser` type definitions,
  `crates/fpas-sema/src/check/decl/types/`, conversion checking,
  `fpas-unit` interfaces, editor highlighting.

## Migration

Rename any identifier spelled `distinct`.

## Documentation

- New distinct-type section in `docs/pascal/language/types/type-aliases.md` or
  a separate page; keyword list; `docs/specs/grammar.ebnf`.

## Verification

- Distinct declarations, unchanged aliases, explicit conversions, rejected
  implicit conversions, swapped domain arguments, arithmetic rejection,
  imported distinct types, `type X = type Y` diagnostic, `distinct` as an
  identifier.
- If AP22.1 is merged: local inference from the explicit conversion.
