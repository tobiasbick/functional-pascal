# `Std.Results`

Helper functions for `Result of T, E` values. See [Error handling](../../language/error-handling/README.md) for the type itself, constructors (`Ok`, `Error`), the `try` operator, and `case` destructuring.

```pascal
program Example;

uses Std.Console as Console;
uses Std.Results as Results;

begin
  var R: result of integer, string := Ok(42);
  Console.WriteLn(Results.Unwrap(R));
end program;
```


## Importing and names

Import with `uses Std.Results as Results;`. Access every exported member through `Results`, for example `Results.Unwrap(...)`. Imports open no short names.

Explicit aliases keep names from different units distinct. Imported routines use alias-qualified calls; receiver-call lookup applies only to routines declared locally.

`Unwrap` and `UnwrapOr` require a `Result of T, E` as their first argument. Passing the wrong container type produces a compile-time type error (`F2006`); use `Std.Options` for the other container type.

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

Examples pass named helper functions whose types match each callback parameter.

---

## `function Unwrap(R: Result of T, E): T`

Extracts the value from `Ok(value)`. **Runtime error** if `R` is `Error`.

```pascal
uses Std.Console as Console;
uses Std.Results as Results;

var R: result of integer, string := Ok(42);
Console.WriteLn(Results.Unwrap(R)); // 42
```

---

## `function UnwrapOr(R: Result of T, E; Default: T): T`

Extracts the value from `Ok(value)`, or returns `Default` if `R` is `Error`.

```pascal
uses Std.Console as Console;
uses Std.Results as Results;

var R: result of integer, string := Error('oops');
Console.WriteLn(Results.UnwrapOr(R, 0)); // 0
```

---

## `function IsOk(R: Result of T, E): boolean`

Returns `true` if `R` is an `Ok` variant.

```pascal
uses Std.Console as Console;
uses Std.Results as Results;

var R: result of integer, string := Ok(42);
Console.WriteLn(Results.IsOk(R)); // true
```

---

## `function IsError(R: Result of T, E): boolean`

Returns `true` if `R` is an `Error` variant.

```pascal
uses Std.Console as Console;
uses Std.Results as Results;

var R: result of integer, string := Error('fail');
Console.WriteLn(Results.IsError(R)); // true
```

---

## `function Map(R: Result of T, E; F: function(V: T): U): Result of U, E`

Transforms the `Ok` value with `F`. If `R` is `Error`, returns it unchanged.

```pascal
uses Std.Conv as Conv;
uses Std.Results as Results;

function DoubleToString(V: integer): string;
begin
  return Conv.IntToStr(V * 2);
end function;

var R: result of integer, string := Ok(21);
var M: result of string, string := Results.Map(R, DoubleToString);
```

---

## `function AndThen(R: Result of T, E; F: function(V: T): Result of U, E): Result of U, E`

Calls `F` with the `Ok` value. `F` returns a new `Result`, enabling chained fallible operations. If `R` is `Error`, returns it unchanged.

```pascal
uses Std.Conv as Conv;
uses Std.Results as Results;

function PositiveToResult(V: integer): result of string, string;
begin
  if V > 0 then
    return Ok(Conv.IntToStr(V));
  else
    return Error('non-positive');
  end if;
end function;

var R: result of integer, string := Ok(10);
var M: result of string, string := Results.AndThen(R, PositiveToResult);
```

---

## `function OrElse(R: Result of T, E; F: function(Err: E): Result of T, F): Result of T, F`

Calls `F` with the `Error` value to attempt recovery. If `R` is `Ok`, returns it unchanged.

```pascal
uses Std.Results as Results;

function RecoverToZero(E: string): result of integer, string;
begin
  return Ok(0);
end function;

var R: result of integer, string := Error('oops');
var M: result of integer, string := Results.OrElse(R, RecoverToZero);
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
