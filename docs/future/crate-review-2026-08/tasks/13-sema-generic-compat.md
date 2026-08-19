# Task 13 — Generic bodies must not treat `T` as every type

Status: complete
Severity: P1
Difficulty: hard
Language gate: no
Depends on: none

## Goal

A generic body cannot return or assign a concrete type that is not `T`. Call-site inference (`Identity(42)`, `Std.Array.Map` callbacks) must keep working.

## Spec

[`docs/pascal/language/types/generics.md`](../../../pascal/language/types/generics.md): type parameters are checked at compile time. `function F<T>(X: T): T` may return `X`, not `'hello'`.

## Bug

`crates/fpas-sema/src/types.rs` `Ty::compatible_with`:

```rust
(Ty::GenericParam(..), _) | (_, Ty::GenericParam(..)) => true,
```

That arm is used by assignment, return, `if`/`while` conditions, and function-type matching. It also exists **on purpose** so `function(X: T): R` can look compatible with `function(X: integer): string` (see the comment on the Function arm in the same file). Call-site inference itself lives in `crates/fpas-sema/src/check/calls.rs` (`collect_type_param_bindings` / `validate_routine_constraints`) and does **not** need the wildcard for ordinary arguments.

So this is **not** “delete the arm and ship”. Deleting it will break Array.Map-style callback checks that still call `compatible_with` on function types containing `T`.

## Required approach

1. Read `compatible_with`, `check_type_compat` (`check/decl/mod.rs`), `check/calls.rs`, and `std_registry/builtins/array/higher_order/callbacks.rs`.
2. Introduce a **stricter** predicate for ordinary assignment/return/condition (`check_type_compat`): `GenericParam` matches only the same parameter name (case-insensitive) or `Error`.
3. Keep a **unification** path for call-site function-typed parameters (either substitute inferred types first, then compare, or keep wildcard only inside that path).
4. Reject `var G: function(X: integer): string := Identity` unless you can prove the spec wants generic-to-concrete function coercion. The review expects reject.

If after reading those files the split is unclear, **stop** and report. Do not land a change that fails existing `generic_methods.rs` or Array Map tests.

## Tests

Add to `crates/fpas-sema/src/tests/` (new file `generic_bodies.rs` or next to `generic_methods.rs`) using `check_errors` / `check_ok`:

Must error (`SEMA_TYPE_MISMATCH`):

```pascal
program T;
function F<T>(X: T): T;
begin
  return 'hello'
end;
begin
end.
```

```pascal
program T;
function F<T>(X: T): T;
begin
  if X then return X;
  return X
end;
begin
end.
```

Must still succeed: existing `Identity` call-site tests; a body `return X` where `X: T`; Array.Map with a concrete callback if covered in std/sema tests (run them).

## Verify

```text
cargo test -p fpas-sema
cargo test -p fpas-std
cargo fmt
```

Run any `tests/stdlib/array/` Map tests you might have affected.

## Done when

- Illegal generic bodies error.
- Inference and higher-order Std callbacks still type-check.
- Docs unchanged.

## Progress

- Split wildcard generic unification from strict ordinary assignment compatibility.
- Free, instance, and static calls now infer type parameters first, substitute them into formal
  parameter types, and then perform strict compatibility checks.
- Added generic-body return/condition, valid inference, and generic-function coercion regressions.
