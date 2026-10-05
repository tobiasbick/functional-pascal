# AP10.1: Typed construction

Package: [AP10: Typed record construction](README.md)

## Scope

Add `TypeName(Field := Value, ...)` for record construction. Record literals
remain valid in this work package.

## Prerequisites

- AP09.1 (named argument syntax and mapping).

## Implementation

- Verify whether a type and a routine may share a name today; make a record
  type name used as a call target always mean construction, and diagnose
  conflicting declarations.
- Sema: named fields only; unknown, duplicate, and missing required fields;
  swapped field types; defaults for omitted fields; private-field
  construction only in the declaring unit. Positional construction is an
  error showing the named form.
- Compiler: evaluate supplied fields in written order, then missing defaults
  in declaration order.
- Generic and imported record types; formatter.

## Affected areas

- `crates/fpas-sema/src/check/decl/types/records.rs`, `check/record_visibility.rs`,
  call checking.
- `crates/fpas-compiler/src/lowering/aggregates/records.rs`.
- `fpas-fmt`, language-service completion inside construction.

## Migration

None in this work package.

## Documentation

- `docs/pascal/language/types/records.md`, `docs/specs/grammar.ebnf`.

## Verification

- Construction with all fields, with defaults, with private fields inside and
  outside the declaring unit, imported types, nested construction.
- Written-order evaluation with side-effect traces.
- Rejections: positional, unknown, duplicate, missing, wrong type.
