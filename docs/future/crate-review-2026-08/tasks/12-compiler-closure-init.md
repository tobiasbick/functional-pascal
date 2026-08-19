# Task 12 — Discover closures in program/unit declaration initializers

Status: in progress
Severity: P1
Difficulty: medium
Language gate: no
Depends on: none

## Goal

`var Increment: procedure() := procedure() begin end;` at program or unit scope compiles. No ICE `unregistered closure`. Bound methods in those initializers also lower.

## Spec

[`docs/pascal/language/functions/closures.md`](../../../pascal/language/functions/closures.md) shows program-level `var Increment: procedure() := procedure() …`. Units may have `var` / `mutable var`.

Sema already accepts these. Lowering does not.

## Bug

`crates/fpas-compiler/src/lowering/mod.rs`: `ClosureRegistry::discover_statements` runs on the program body and each named routine. Root then lowers `Decl::Var` / `MutableVar` / `Const` initializers **before** the body. Units pass `&[]` as the body (`lower_unit`). Miss → `crates/fpas-compiler/src/lowering/expr.rs` ICE `"unregistered closure"`. Bound methods miss `bound_targets`.

Function-local `var` **statements** inside `begin` are already visited.

## Fix

Walk declaration initializers (program and unit) in `discover_statements` / a sibling `discover_decls` **before** lowering those initializers. Reuse the same visitor that already finds closures in statements. Cover `const` initializers too if they can hold closures.

Do not register the same closure twice. Do not change capture semantics.

## Tests

`crates/fpas-compiler/src/tests/closures.rs` (create or extend) with `assert_succeeds`:

```pascal
program ClosureInit;
var F: procedure() := procedure() begin end;
begin
  F()
end.
```

If bound-method init is one extra test and the syntax is already used in `aggregates.rs`, add `var F := C.Add` style only if that is valid at program scope; otherwise skip bound methods.

Unit-level: only if compiler tests can compile a unit. If not, program-level is enough for this task.

## Verify

```text
cargo test -p fpas-compiler
cargo fmt
```

## Done when

- Program-level closure init runs.
- No `"unregistered closure"` on that shape.
- Docs unchanged.

## Progress

- Base commit: 74b16b7b
- Current step: verify declaration-initializer closure discovery before root lowering
- Files changed: compiler closure registry/lowering and closure tests
- Verification: not run
- Blockers: none
