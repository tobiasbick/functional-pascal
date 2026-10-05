# AP03.1: Migrate closed-enum catch-alls

Package: [AP03: Explicit closed-enum cases](README.md)

## Scope

Replace every `case ... else` over a closed enum in the repository with a full
variant list or an `is` test, while `else` is still accepted.

## Prerequisites

- AP20.3 (`is` test) for sites that handle a single variant.
- AP13.5 (`when` arms with `null;`) so migrated arms are written once in the
  final syntax.

## Implementation

- Inventory `case ... else` sites over closed enums using semantic type
  information, not text search. Large enums at planning time: `TuiElement`
  (43 variants), `TuiMsg` (37), `TuiStyleRole` (34), `KeyKind` (29).
- Rewrite each site to a full variant list (grouping no-op variants in one
  `null;` arm) or to an `is` test when only one variant is handled.
- Preserve behavior: the former `else` body moves to exactly the variants it
  previously covered.

## Affected areas

- `.fpas` sources under `lib/`, `apps/`, `examples/`, `tests/`; Rust-embedded
  fixtures; documentation examples.

## Migration

This work package is the migration.

## Documentation

Update documentation examples that use `else` over a closed enum. Language
rules are unchanged.

## Verification

- A temporary semantic check on the branch reports zero closed-enum catch-alls.
- `fpas test tests/suite.fpasprj` and example/app checks pass unchanged.
