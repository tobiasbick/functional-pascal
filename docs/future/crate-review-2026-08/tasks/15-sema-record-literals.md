# Task 15 — Contextually type record literals everywhere

Status: open
Severity: P1
Difficulty: medium
Language gate: no
Depends on: none (complete this before deciding task 14)

## Goal

Omitted fields with defaults work in assignment, call arguments, array-of-record initializers, `const`, and `return`, not only in `var` initializers that are themselves `record … end`.

## Spec

[`docs/pascal/language/types/records.md`](../../../pascal/language/types/records.md): when a record literal omits a field that has a default, the compiler substitutes the default. Fields without a default stay required.

## Bug

Sema applies defaults only when a `var` initializer is a record literal (`crates/fpas-sema/src/check/decl/vars.rs`). Other positions go through `check_expr` → anonymous `{X}` then `compatible_with` against the named type, which fails if a defaulted field is missing.

`return` calls `try_annotate_expected_record_literals` **after** `check_type_compat`, so it is still rejected. Call arguments are never annotated; lowering looks up layouts by record **name**, so `Draw(record X := 1; Y := 2; end)` is typed `"<anonymous>"` while the parameter is `Point`.

## Fix

Apply expected-type annotation **before** compatibility checks for:

- assignment RHS
- `const` initializer
- `return`
- call arguments (parameter type)
- array elements when the array type is known

Reuse `try_annotate_expected_record_literals` (find it and call it earlier). Fill defaulted fields in the typed literal so lowering sees the named type.

Do not allow omitting required fields.

## Tests

`crates/fpas-sema/src/tests/` plus a compiler `assert_succeeds` if sema-only would miss lowering:

```pascal
program T;
type Point = record
  X: integer := 0;
  Y: integer := 0;
end;
function Origin(): Point;
begin
  return record X := 0; end
end;
procedure Draw(P: Point);
begin
end;
begin
  var P: Point;
  P := record X := 1; end;
  Draw(record X := 1; Y := 2; end)
end.
```

`check_ok` for the above. A literal missing a required (non-default) field still errors.

## Verify

```text
cargo test -p fpas-sema
cargo test -p fpas-compiler
cargo fmt
```

## Done when

- Defaulted omissions work outside `var`.
- Required fields still required.
- Docs unchanged.
