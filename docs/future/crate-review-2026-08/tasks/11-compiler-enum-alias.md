# Task 11 — Simple enum members lower to backing integers, not variant index

Status: open
Severity: P1
Difficulty: medium
Language gate: no
Depends on: none

## Goal

`PaletteColor.Green` and imported `Color.Red` emit the declared backing integer, same as `Color.Green` on the original typedef.

## Spec

[`docs/pascal/language/types/enums.md`](../../../pascal/language/types/enums.md) and [`docs/pascal/language/types/type-aliases.md`](../../../pascal/language/types/type-aliases.md). Sema already accepts alias-qualified variants.

## Bug

`crates/fpas-compiler/src/lowering/mod.rs` `collect_enum_constants` only inserts `{TypeDef}.{member}` and unambiguous shorts from **local** `TypeBody::Enum` bodies.

`crates/fpas-compiler/src/lowering/expr/designators.rs`: on map miss it uses `variants.iter().position(...)` as `Const(Integer(index))`. Wrong when the qualified name is not the original typedef: `type PaletteColor = Color;` then `PaletteColor.Green` with `Green = 20` becomes `1`.

Existing test `simple_enum_values_keep_backing_numbers_and_case_insensitivity` in `crates/fpas-compiler/src/tests/aggregates.rs` only uses `State.Running` (hits the map).

## Fix

When lowering a simple (non-data) enum designator, take the backing value from the **enum type’s variant**, not from the constant map keyed by the source spelling. The `Ty::Enum` from `expression_type` already has variants; those variants should carry the integer value (check the `EnumTy` / variant struct). Prefer that over expanding `collect_enum_constants`.

If variants do not store backing integers, add them in sema’s enum type **or** look up the original typedef name from the alias and use the existing map. Do not emit `position` as a value except when it equals the implicit backing (0,1,2,…) — even then prefer the stored integer.

Do not change data enums (`MakeEnum`).

## Tests

Add next to `simple_enum_values_keep_backing_numbers_and_case_insensitivity` in `crates/fpas-compiler/src/tests/aggregates.rs` using `assert_succeeds`:

```pascal
program AliasEnum;
type
  Color = enum
    Red;
    Green = 20;
    Blue;
  end;
  PaletteColor = Color;
begin
  if PaletteColor.Green <> 20 then panic('alias backing')
end.
```

If compiler tests cannot see type aliases that way, use whatever alias syntax the compiler tests already use. Also add a case that compares `Green` via the original name still equals 20.

Imported enums: only add a compiler/unit test if this crate already has a multi-file compile helper. If not, skip import coverage here and mention it in the summary (sema+project test would belong in `fpas-cli` / `tests/`). Do not build a new project harness.

## Verify

```text
cargo test -p fpas-compiler
cargo fmt
```

## Done when

- Alias-qualified simple enum members use backing values.
- Data-enum lowering unchanged.
- Docs unchanged.
