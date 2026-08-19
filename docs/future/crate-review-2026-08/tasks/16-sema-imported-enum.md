# Task 16 — Ambiguous imported enum type makes `Color.Red` ambiguous

Status: complete
Severity: P1
Difficulty: medium
Language gate: no
Depends on: none

## Goal

When two imports share the short type name `Color`, `Color.Red` is `SEMA_AMBIGUOUS_IMPORTED_NAME` (same as unqualified clash). `UnitA.Color.Red` still works.

## Spec

[`docs/pascal/program-structure/units.md`](../../../pascal/program-structure/units.md): no error at `uses`; error when the ambiguous short name is used. [`docs/pascal/language/types/enums.md`](../../../pascal/language/types/enums.md) covers ambiguous **variant** shorts; this task is the **type** short plus `.Red`.

## Bug

`crates/fpas-sema/src/interface/install.rs` `install_imported_enum_variants` always `define_in_root("Color.Red")`. A failed second insert is ignored. Short `Color` is marked ambiguous, but `Color.Red` stays bound to whichever interface was installed first.

## Fix

If the short type name is ambiguous, do not leave `Color.Red` as a unique root binding. Mark `Color.Red` ambiguous too, or resolve `Color.Red` by looking up `Color` first (which is already ambiguous). Qualified `OtherUnit.Color.Red` must remain unique.

Existing tests cover ambiguous **functions** (`Value`). Mirror that pattern for two exported enums with the same short name.

## Tests

Follow `crates/fpas-sema/src/tests/integration/` / `interface/tests.rs` for two-unit fixtures. If interface tests already load two units, add:

- `var C: Color := Color.Red` → `SEMA_AMBIGUOUS_IMPORTED_NAME`
- `var C: UnitA.Color := UnitA.Color.Red` → ok

## Verify

```text
cargo test -p fpas-sema
cargo fmt
```

## Done when

- `Color.Red` is ambiguous when `Color` is.
- Qualified names work.
- Docs unchanged.

## Progress

- Base commit: 74b16b7b
- Current step: verify ambiguous and fully qualified imported enum variants
- Files changed: semantic interface installation and interface tests
- Verification: full workspace definition of done passed on 2026-08-19
- Blockers: none
