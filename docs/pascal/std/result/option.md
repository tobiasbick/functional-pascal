# `Std.Options`

Helper functions for `Option of (T)` values. See [Error handling](../../language/error-handling/README.md) for the type itself, constructors (`Option.Some`, `Option.None`), the `try` operator, and `case` destructuring.

```pascal
program Example;

uses Std.Console as Console;
uses Std.Options as Options;

begin
  const O: option of (integer) := Option.Some(7);
  Console.WriteLn(Options.Unwrap(O));
end program;
```


## Importing and names

Import with `uses Std.Options as Options;`. Access every exported member through `Options`, for example `Options.Unwrap(...)`. Imports open no short names.

Explicit aliases keep names from different units distinct. Imported routines use alias-qualified calls; receiver-call lookup applies only to routines declared locally.

`Unwrap` and `UnwrapOr` require a `Option of (T)` as their first argument. Passing the wrong container type produces a compile-time type error (`F2006`); use `Std.Results` for the other container type.

---

## Quick reference

| Kind | Name | Notes |
|------|------|--------|
| function | `Unwrap(O: Option of (T)): T` | panics if None |
| function | `UnwrapOr(O: Option of (T); Default: T): T` | returns Default if None |
| function | `IsSome(O: Option of (T)): boolean` | true if Some |
| function | `IsNone(O: Option of (T)): boolean` | true if None |
| function | `Map(O: Option of (T); F: pure function(V: T): U): Option of (U)` | transform Some value |
| function | `AndThen(O: Option of (T); F: pure function(V: T): Option of (U)): Option of (U)` | chain fallible operations |
| function | `OrElse(O: Option of (T); F: pure function(): Option of (T)): Option of (T)` | provide fallback |

---

Examples pass named helper functions whose types match each callback parameter.

---

## `pure function Unwrap(O: Option of (T)): T`

Extracts the value from `Option.Some(value)`. **Runtime error** if `O` is `Option.None`.

```pascal
uses Std.Console as Console;
uses Std.Options as Options;

const O: option of (integer) := Option.Some(7);
Console.WriteLn(Options.Unwrap(O)); // 7
```

---

## `pure function UnwrapOr(O: Option of (T); Default: T): T`

Extracts the value from `Option.Some(value)`, or returns `Default` if `O` is `Option.None`.

```pascal
uses Std.Console as Console;
uses Std.Options as Options;

const O: option of (integer) := Option.None;
Console.WriteLn(Options.UnwrapOr(O, -1)); // -1
```

---

## `pure function IsSome(O: Option of (T)): boolean`

Returns `true` if `O` is a `Option.Some` variant.

```pascal
uses Std.Console as Console;
uses Std.Options as Options;

const O: option of (integer) := Option.Some(7);
Console.WriteLn(Options.IsSome(O)); // true
```

---

## `pure function IsNone(O: Option of (T)): boolean`

Returns `true` if `O` is `Option.None`.

```pascal
uses Std.Console as Console;
uses Std.Options as Options;

const O: option of (integer) := Option.None;
Console.WriteLn(Options.IsNone(O)); // true
```

---

## `pure function Map(O: Option of (T); F: pure function(V: T): U): Option of (U)`

Transforms the `Option.Some` value with `F`. If `O` is `Option.None`, returns `Option.None`.

```pascal
uses Std.Conv as Conv;
uses Std.Options as Options;

pure function TripleToString(V: integer): string;
begin
  return Conv.IntToStr(V * 3);
end function;

const O: option of (integer) := Option.Some(7);
const M: option of (string) := Options.Map(O, TripleToString);
```

---

## `pure function AndThen(O: Option of (T); F: pure function(V: T): Option of (U)): Option of (U)`

Calls `F` with the `Option.Some` value. `F` returns a new `Option`, enabling chained lookups. If `O` is `Option.None`, returns `Option.None`.

```pascal
uses Std.Conv as Conv;
uses Std.Options as Options;

pure function PositiveToOptionString(V: integer): option of (string);
begin
  if V > 0 then
    return Option.Some(Conv.IntToStr(V));
  else
    return Option.None;
  end if;
end function;

const O: option of (integer) := Option.Some(5);
const M: option of (string) := Options.AndThen(O, PositiveToOptionString);
```

---

## `pure function OrElse(O: Option of (T); F: pure function(): Option of (T)): Option of (T)`

Calls `F` to provide a fallback when `O` is `Option.None`. If `O` is `Option.Some`, returns it unchanged.

```pascal
uses Std.Options as Options;

pure function Fallback99(): option of (integer);
begin
  return Option.Some(99);
end function;

const O: option of (integer) := Option.None;
const M: option of (integer) := Options.OrElse(O, Fallback99);
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
