# `Std.Result`

Helper functions for `Result of T, E` values. See [07-error-handling.md](../07-error-handling.md) for the type itself, constructors (`Ok`, `Error`), the `try` operator, and `case` destructuring.

```pascal
program Example;
uses Std.Console, Std.Result;
begin
  var R: Result of integer, string := Ok(42);
  WriteLn(Unwrap(R))
end.
```

**Maintenance (implementers only):** align with [`std_registry/loaded/result_option.rs`](../../../crates/fpas-sema/src/std_registry/loaded/result_option.rs), [`std_registry/builtins/result_option.rs`](../../../crates/fpas-sema/src/std_registry/builtins/result_option.rs), [`result_option.rs`](../../../crates/fpas-std/src/result_option.rs), [`intrinsics.rs`](../../../crates/fpas-std/src/intrinsics.rs), and [`intrinsic.rs`](../../../crates/fpas-bytecode/src/intrinsic.rs).

---

## Importing and names

After `uses Std.Result;` use short names (`Unwrap`, `IsOk`, …) or qualified (`Std.Result.Unwrap`, …).

**Ambiguity with `Std.Option`:** the short names **`Unwrap`** and **`UnwrapOr`** clash with `Std.Option`. When both units are imported, qualify as `Std.Result.Unwrap(R)` vs `Std.Option.Unwrap(O)`.

---

## Quick reference

| Kind | Name | Notes |
|------|------|--------|
| function | `Unwrap(R: Result of T, E): T` | panics if Error |
| function | `UnwrapOr(R: Result of T, E; Default: T): T` | returns Default if Error |
| function | `IsOk(R: Result of T, E): boolean` | true if Ok |
| function | `IsError(R: Result of T, E): boolean` | true if Error |

---

## `function Unwrap(R: Result of T, E): T`

Extracts the value from `Ok(value)`. **Runtime error** if `R` is `Error`.

```pascal
var R: Result of integer, string := Ok(42);
WriteLn(Unwrap(R))                             { 42 }
```

---

## `function UnwrapOr(R: Result of T, E; Default: T): T`

Extracts the value from `Ok(value)`, or returns `Default` if `R` is `Error`.

```pascal
var R: Result of integer, string := Error('oops');
WriteLn(UnwrapOr(R, 0))                       { 0 }
```

---

## `function IsOk(R: Result of T, E): boolean`

Returns `true` if `R` is an `Ok` variant.

```pascal
var R: Result of integer, string := Ok(42);
WriteLn(IsOk(R))                               { true }
```

---

## `function IsError(R: Result of T, E): boolean`

Returns `true` if `R` is an `Error` variant.

```pascal
var R: Result of integer, string := Error('fail');
WriteLn(IsError(R))                              { true }
```

---

## Implementation map (contributors)

| Concern | Location |
|---------|-----------|
| Runtime logic | [`result_option.rs`](../../../crates/fpas-std/src/result_option.rs) |
| Type checking | [`std_registry/builtins/result_option.rs`](../../../crates/fpas-sema/src/std_registry/builtins/result_option.rs) |
| Registration | [`std_registry/loaded/result_option.rs`](../../../crates/fpas-sema/src/std_registry/loaded/result_option.rs) |

[← Standard library index](README.md)
