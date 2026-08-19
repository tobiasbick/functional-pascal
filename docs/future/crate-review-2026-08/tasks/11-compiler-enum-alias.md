# Task 11 — Preserve simple-enum backing values through aliases and imports

Status: complete
Severity: P1
Difficulty: medium
Language gate: no
Depends on: none

## Goal

Every spelling of a simple enum member emits its declared backing integer. This includes the
original type, a type alias, an imported type, and fully qualified imported names.

## Contract

- [`Enums`](../../../pascal/language/types/enums.md)
- [`Type aliases`](../../../pascal/language/types/type-aliases.md)
- [`Compiled units`](../../../pascal/program-structure/units.md#compiled-unit-sidecars)

## Verified cause

`lowering/mod.rs::collect_enum_constants` records backing integers only from local enum AST nodes
and only under the original local spelling. On a miss,
`lowering/expr/designators.rs::lower_designator_expression` uses the variant position.

`fpas_unit::interface::EnumVariant` already stores `backing_value`, and source interface export
populates it. The loss occurs in `fpas-sema`: `EnumVariantTy` has no backing-value field,
`interface_to_enum` discards the persisted value, and local semantic enum construction also omits
it. Lowering therefore cannot recover the resolved value from the semantic type.

## Required implementation

1. Add an explicit optional/simple-enum backing value to `EnumVariantTy`. Data-carrying variants
   must remain distinct and must not acquire ordinal-value semantics.
2. Refactor local enum checking so validation and semantic construction share one backing-value
   calculation. Populate implicit as well as explicit values. Register intrinsic simple enums with
   their ordinal values and data enums without one.
3. Copy the existing interface `backing_value` in both `enum_to_interface` and `interface_to_enum`.
   Keep `fpas-unit`'s existing schema and serialization unchanged; add/adjust conversion tests so
   aliases and imported interfaces cannot drop the field.
4. Lower simple members from the resolved semantic variant value. Remove the variant-position
   fallback for simple enums rather than retaining two authorities.
5. Keep `collect_enum_constants` only for constants it still owns; remove dead enum-specific logic
   exposed by the change.

Do not change data-enum `MakeEnum` lowering.

## Tests

- Compiler: implicit and explicit values through original and alias-qualified names.
- Sema/interface conversion: imported simple variant and exported enum alias retain backing values.
- Multi-unit compiler/build test: imported qualified enum member with a non-ordinal value executes
  as that declared value.
- Existing data-enum and sidecar tests remain green.

## Verify

```text
cargo test -p fpas-sema
cargo test -p fpas-compiler
cargo build
cargo test --workspace
cargo fmt
```

## Done when

- No simple-enum spelling derives its value from `variants.position()`.
- Backing values survive semantic interface conversion; the existing sidecar field remains intact.
- Local, alias-qualified, and imported regressions pass.
- Current language docs remain unchanged.

## Progress

- Added simple-enum backing values to semantic variants; data-enum variants retain `None`.
- Local checking now calculates validated implicit/explicit values once for both diagnostics and
  semantic construction; intrinsic simple enums receive ordinal values.
- Interface conversion copies the existing sidecar field in both directions, including aliases.
- Compiler designator lowering now reads the resolved semantic variant value and no longer falls
  back to declaration position or a local AST enum-constant table.
- Added local original/alias execution coverage and export/import/alias interface coverage.
