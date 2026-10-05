# `Std.Results`

Helper functions for `Result of (T, E)` values. See [Error handling](../../language/error-handling/README.md) for the type itself, constructors (`Result.Ok`, `Result.Error`), the `try` operator, and `case` destructuring.

```pascal
program Example;

uses Std.Console as Console;
uses Std.Results as Results;

begin
  const R: result of (integer, string) := Result.Ok(42);
  Console.WriteLn(Results.Unwrap(R));
end program;
```


## Importing and names

Import with `uses Std.Results as Results;`. Access every exported member through `Results`, for example `Results.Unwrap(...)`. Imports open no short names.

Explicit aliases keep names from different units distinct. Imported routines use alias-qualified calls.

`Unwrap` and `UnwrapOr` require a `Result of (T, E)` as their first argument. Passing the wrong container type produces a compile-time type error (`F2006`); use `Std.Options` for the other container type.

---

## Quick reference

| Kind | Name | Notes |
|------|------|--------|
| function | `Unwrap(R: Result of (T, E)): T` | panics if Error |
| function | `UnwrapOr(R: Result of (T, E); Default: T): T` | returns Default if Error |
| function | `IsOk(R: Result of (T, E)): boolean` | true if Ok |
| function | `IsError(R: Result of (T, E)): boolean` | true if Error |
| function | `Map(R: Result of (T, E); F: pure function(V: T): U): Result of (U, E)` | transform Ok value |
| function | `AndThen(R: Result of (T, E); F: pure function(V: T): Result of (U, E)): Result of (U, E)` | chain fallible operations |
| function | `OrElse(R: Result of (T, E); F: pure function(Err: E): Result of (T, F)): Result of (T, F)` | recover from Error |

---

Examples pass named helper functions whose types match each callback parameter.

---

## `pure function Unwrap(R: Result of (T, E)): T`

Extracts the value from `Result.Ok(value)`. **Runtime error** if `R` is `Result.Error`.

```pascal
uses Std.Console as Console;
uses Std.Results as Results;

const R: result of (integer, string) := Result.Ok(42);
Console.WriteLn(Results.Unwrap(R)); // 42
```

---

## `pure function UnwrapOr(R: Result of (T, E); Default: T): T`

Extracts the value from `Result.Ok(value)`, or returns `Default` if `R` is `Result.Error`.

```pascal
uses Std.Console as Console;
uses Std.Results as Results;

const R: result of (integer, string) := Result.Error('oops');
Console.WriteLn(Results.UnwrapOr(R, 0)); // 0
```

---

## `pure function IsOk(R: Result of (T, E)): boolean`

Returns `true` if `R` is an `Result.Ok` variant.

```pascal
uses Std.Console as Console;
uses Std.Results as Results;

const R: result of (integer, string) := Result.Ok(42);
Console.WriteLn(Results.IsOk(R)); // true
```

---

## `pure function IsError(R: Result of (T, E)): boolean`

Returns `true` if `R` is an `Result.Error` variant.

```pascal
uses Std.Console as Console;
uses Std.Results as Results;

const R: result of (integer, string) := Result.Error('fail');
Console.WriteLn(Results.IsError(R)); // true
```

---

## `pure function Map(R: Result of (T, E); F: pure function(V: T): U): Result of (U, E)`

Transforms the `Result.Ok` value with `F`. If `R` is `Result.Error`, returns it unchanged.

```pascal
uses Std.Conv as Conv;
uses Std.Results as Results;

pure function DoubleToString(V: integer): string;
begin
  return Conv.IntToStr(V * 2);
end function;

const R: result of (integer, string) := Result.Ok(21);
const M: result of (string, string) := Results.Map(R, DoubleToString);
```

---

## `pure function AndThen(R: Result of (T, E); F: pure function(V: T): Result of (U, E)): Result of (U, E)`

Calls `F` with the `Result.Ok` value. `F` returns a new `Result`, enabling chained fallible operations. If `R` is `Result.Error`, returns it unchanged.

```pascal
uses Std.Conv as Conv;
uses Std.Results as Results;

pure function PositiveToResult(V: integer): result of (string, string);
begin
  if V > 0 then
    return Result.Ok(Conv.IntToStr(V));
  else
    return Result.Error('non-positive');
  end if;
end function;

const R: result of (integer, string) := Result.Ok(10);
const M: result of (string, string) := Results.AndThen(R, PositiveToResult);
```

---

## `pure function OrElse(R: Result of (T, E); F: pure function(Err: E): Result of (T, F)): Result of (T, F)`

Calls `F` with the `Result.Error` value to attempt recovery. If `R` is `Result.Ok`, returns it unchanged.

```pascal
uses Std.Results as Results;

pure function RecoverToZero(E: string): result of (integer, string);
begin
  return Result.Ok(0);
end function;

const R: result of (integer, string) := Result.Error('oops');
const M: result of (integer, string) := Results.OrElse(R, RecoverToZero);
```

---

## Implementation (contributors)

| Concern | Location |
|---------|-----------|
| Runtime logic | [`result_option.rs`](../../../../crates/fpas-std/src/result_option.rs) |
| Type checking | [`std_registry/builtins/result_option.rs`](../../../../crates/fpas-sema/src/std_registry/builtins/result_option.rs) |
| Registration | [`std_registry/loaded/result_option.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/result_option.rs) |

## See also

- [Result and Option index](README.md)
- [Standard library index](../README.md)
