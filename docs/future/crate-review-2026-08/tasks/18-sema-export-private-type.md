# Task 18 — Reject public signatures that name a private type

Status: open
Severity: P2
Difficulty: medium
Language gate: no
Depends on: none

## Goal

`public function Make(): Hidden` where `Hidden` is a private record/enum/alias is a sema error at the **exporting** unit. Importers must not receive an unusable `Named` they cannot resolve.

## Spec

[`docs/pascal/program-structure/visibility.md`](../../../pascal/program-structure/visibility.md) and units docs: only `public` declarations are exported. A public API that mentions a non-exported type is not usable and should be diagnosed.

## Bug

`crates/fpas-sema/src/interface/export.rs`: any `public` declaration is exported. `ty_to_interface_reference` + `qualify_owned_name` names private types because they sit in `own_types`. `Hidden` itself is not exported.

## Fix

When exporting a public declaration, walk its signature types. If a named type is owned by this unit and not public, emit a diagnostic (reuse or add `SEMA_*` code with a hint: make the type `public` or stop exporting the routine). Do not export the private type as a side effect.

Do not introduce a `private` keyword.

## Tests

Interface / integration test: unit with private `Hidden` and `public function Make(): Hidden` → error. Control: public type in the signature exports as today.

## Verify

```text
cargo test -p fpas-sema
cargo fmt
```

## Done when

- The leaking signature is a compile error.
- Valid public types in public signatures still export.
- Docs: add a sentence under visibility/units **only if** this rule is not already stated; otherwise unchanged.
