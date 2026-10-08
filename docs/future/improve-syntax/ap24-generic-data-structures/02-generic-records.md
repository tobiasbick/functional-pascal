# AP24.2: Generic records

Package: [AP24: Generic data structures](README.md)

## Scope

Allow user-defined generic records, for example
`type Pair of (K: Comparable, V) = record ... end record;`, with typed
construction and constructor type-argument inference.

AP10.1 supplies construction for concrete record types and their aliases.
This work package introduces generic record declarations and extends that
construction with generic type substitution and type-argument inference.

## Prerequisites

- AP24.1 (parenthesized type arguments).
- AP10.1 (typed construction).

## Implementation

- Parser: type parameters after the type name with `of`.
- Sema: generic record types, substitution, constraints reusing routine
  constraints, equality, private fields, type-argument inference from field
  values or the expected type with an annotation error otherwise.
- Compiler, unit interfaces, and linker support generic record layouts across
  compiled units.
- Language service: hover and completion for instantiated fields.

## Affected areas

- `crates/fpas-parser/src/parser/decl/data/type_defs.rs`,
  `crates/fpas-sema/src/check/name_resolution/types/generics.rs`,
  `check/decl/types/`, `crates/fpas-compiler/src/lowering/types/`,
  `fpas-unit` interfaces.

## Migration

None.

## Documentation

- `docs/pascal/language/types/generics.md`, `records.md`,
  `docs/specs/grammar.ebnf`.

## Verification

- Several concrete instantiations, nesting, constraints, invalid type
  arguments, underconstrained construction, imported generic records,
  equality.
