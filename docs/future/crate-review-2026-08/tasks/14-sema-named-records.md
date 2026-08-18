# Task 14 — Named record types compare by identity

Status: open
Severity: P1
Difficulty: medium
Language gate: no
Depends on: none

## Goal

Two different named record types are not interchangeable even if their fields match. `Point` is not `Size`.

## Spec

[`docs/pascal/language/types/records.md`](../../../pascal/language/types/records.md) declares named types. There is no structural-equivalence rule for distinct names. Recursive/`Named` matching by **name** already exists in `compatible_with`.

## Bug

`crates/fpas-sema/src/types.rs` record arm: if both `private_members` lists are empty, compare field shapes and **ignore names**. Program-local records always get `private_members = []` (`crates/fpas-sema/src/check/decl/types/records.rs`). `Point` and `Size` with `X,Y: real` assign. A method-less value can be assigned to a type that has methods.

Private unit records are nominal only when `private_members` is non-empty; two different private types with the same fields can still merge via `private_records_are_compatible_inside_owner` — do not break same-type private access inside the owner.

## Fix

Compare record types by name (case-insensitive) **and** owner when present, like the private-members branch. Keep field-shape comparison only for **anonymous** record types (name `"<anonymous>"` or whatever the checker uses today for `record X := 1; Y := 2; end` literals). Confirm the anonymous name in `Ty::Record` before special-casing.

Do not make two anonymous literals with the same fields incompatible if that would reject `var P: Point := record X := 1.0; Y := 2.0; end` — that path should use expected-type annotation (task 15), not structural mixing of two named types.

## Tests

`crates/fpas-sema/src/tests/decl/types.rs` or `stmt/assignment.rs`:

```pascal
program T;
type
  Point = record X: real; Y: real; end;
  Size = record X: real; Y: real; end;
begin
  var P: Point := record X := 1.0; Y := 2.0; end;
  var S: Size := P
end.
```

Expect `SEMA_TYPE_MISMATCH`. Same-type assignment still `check_ok`.

## Verify

```text
cargo test -p fpas-sema
cargo fmt
```

## Done when

- Distinct named records are incompatible.
- Same named type still assigns.
- Anonymous literals vs a matching named type: if this task makes `var P: Point := record … end` fail, stop and coordinate with task 15 (contextual typing) instead of forcing structural named equality back in.
