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
| function | `Map(R: Result of T, E; F: function(V: T): U): Result of U, E` | transform Ok value |
| function | `AndThen(R: Result of T, E; F: function(V: T): Result of U, E): Result of U, E` | chain fallible operations |
| function | `OrElse(R: Result of T, E; F: function(Err: E): Result of T, F): Result of T, F` | recover from Error |

---

Examples below use named helper functions. Anonymous function expressions are not supported.

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

## `function Map(R: Result of T, E; F: function(V: T): U): Result of U, E`

Transforms the `Ok` value with `F`. If `R` is `Error`, returns it unchanged.

```pascal
function DoubleToString(V: integer): string;
begin
  return IntToStr(V * 2)
end;

var R: Result of integer, string := Ok(21);
var M: Result of string, string := Map(R, DoubleToString);
{ M = Ok('42') }
```

---

## `function AndThen(R: Result of T, E; F: function(V: T): Result of U, E): Result of U, E`

Calls `F` with the `Ok` value. `F` returns a new `Result`, enabling chained fallible operations. If `R` is `Error`, returns it unchanged.

```pascal
function PositiveToResult(V: integer): Result of string, string;
begin
  if V > 0 then return Ok(IntToStr(V))
  else return Error('non-positive')
end;

var R: Result of integer, string := Ok(10);
var M: Result of string, string := AndThen(R, PositiveToResult);
{ M = Ok('10') }
```

---

## `function OrElse(R: Result of T, E; F: function(Err: E): Result of T, F): Result of T, F`

Calls `F` with the `Error` value to attempt recovery. If `R` is `Ok`, returns it unchanged.

```pascal
function RecoverToZero(E: string): Result of integer, string;
begin
  return Ok(0)
end;

var R: Result of integer, string := Error('oops');
var M: Result of integer, string := OrElse(R, RecoverToZero);
{ M = Ok(0) }
```

---

## Implementation map (contributors)

| Concern | Location |
|---------|-----------|
| Runtime logic | [`result_option.rs`](../../../crates/fpas-std/src/result_option.rs) |
| Type checking | [`std_registry/builtins/result_option.rs`](../../../crates/fpas-sema/src/std_registry/builtins/result_option.rs) |
| Registration | [`std_registry/loaded/result_option.rs`](../../../crates/fpas-sema/src/std_registry/loaded/result_option.rs) |

[← Standard library index](README.md)
