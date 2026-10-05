# AP13.3: Named closers for declarations

Package: [AP13: Explicit block boundaries](README.md)

## Scope

Introduce the named closers for declarations: `end function;`,
`end procedure;` (including methods and nested routines), `end record;`,
`end enum;`, and `end unit;`. The program keeps `begin ... end.`.

## Prerequisites

- AP13.2 (statement terminators).

## Implementation

- Parser: require the matching closer for each declaration kind; report
  mismatched closers with the expected one and recover.
- Units end with `end unit;` after their last declaration.
- Formatter: emit the closers.

## Affected areas

- `crates/fpas-parser/src/parser/program.rs`, `decl/routines.rs`,
  `decl/data/type_defs.rs`.
- `crates/fpas-fmt/src/emit/decl/`, `emit/program.rs`.
- Generated `lib/api/` declarations and their generator; CLI templates;
  editor snippets and indentation rules.

## Migration

All repository consumers, including generated declarations and documentation
examples.

## Documentation

- `docs/specs/grammar.ebnf` (`function_decl`, `procedure_decl`,
  `record_type`, `enum_type`, `unit`), routine, record, enum, and unit pages,
  `fmt-style.md`.

## Verification

- Each closer in units and programs, nested routines, methods, records with
  methods, enums; mismatched and missing closers; comments before closers.
