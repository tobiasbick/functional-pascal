# AP24.1: Parenthesized type argument lists

Package: [AP24: Generic data structures](README.md)

## Scope

Change multi-argument type applications to the parenthesized form, for example
`Result of T, E` to `Result of (T, E)`. Single-argument applications and
`dict of K to V` are unchanged.

## Prerequisites

None.

## Implementation

- Parser: parenthesized type argument lists after `of`; nested applications
  such as `Result of (Option of User, string)`.
- Diagnose the unparenthesized multi-argument form and angle-bracket
  applications with the `of` form.
- Formatter and generated declarations.

## Affected areas

- `crates/fpas-parser/src/parser/decl/type_expr.rs`, `fpas-fmt`, the intrinsic
  declaration generator for `lib/api/`.

## Migration

About 890 `Result of T, E` sites existed in `.fpas` sources at planning time;
migrate them and all other consumers, including generated declarations,
fixtures, and documentation.

## Documentation

- `docs/specs/grammar.ebnf` (`type_expr`),
  `docs/pascal/language/types/result-option-types.md`, `generics.md`,
  affected `Std` pages.

## Verification

- Parser tests for single, multiple, and nested arguments and both rejected
  forms; corpus round trip; FPAS suite.
