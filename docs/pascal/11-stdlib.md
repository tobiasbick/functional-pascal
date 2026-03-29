# 11. Standard Library

The standard library provides built-in units under the reserved `Std` namespace. Import them via `uses` and refer to symbols by short name or fully qualified name:

```pascal
program Hello;

uses
  Std.Console,
  Std.Math;

begin
  WriteLn(Sqrt(16.0));                           { short name }
  Std.Console.WriteLn(Std.Math.Sqrt(16.0));      { fully qualified }
end.
```

See [09-units.md](09-units.md) for `uses` rules, short name ambiguity, and the reserved `Std` namespace. Full per-unit reference with examples and edge cases lives in [std/](std/README.md).

---

## Unit overview

| Unit | Purpose | Key symbols | Details |
|------|---------|-------------|---------|
| [`Std.Console`](std/console.md) | Console I/O, CRT screen control | `Write`, `WriteLn`, `ReadLn`, `ClrScr`, `GotoXY`, … | [console.md](std/console.md) |
| [`Std.Str`](std/str.md) | String operations | `Length`, `ToUpper`, `Contains`, `Split`, `Join`, `PadLeft`, `CharAt`, `Ord`, `Chr`, … | [str.md](std/str.md) |
| [`Std.Conv`](std/conv.md) | Type conversions | `IntToStr`, `StrToInt`, `RealToStr`, `BoolToStr`, `IntToHex`, … | [conv.md](std/conv.md) |
| [`Std.Math`](std/math.md) | Mathematical functions | `Pi`, `Abs`, `Sqrt`, `Pow`, `Sin`, `Cos`, `Tan`, `Log`, `Exp`, `Clamp`, `Random`, … | [math.md](std/math.md) |
| [`Std.Array`](std/array.md) | Array helpers | `Length`, `Push`, `Pop`, `Sort`, `Slice`, `Map`, `Filter`, `Reduce`, `Find`, `Any`, `All`, … | [array.md](std/array.md) |
| [`Std.Dict`](std/dict.md) | Dictionary helpers | `Length`, `ContainsKey`, `Keys`, `Values`, `Remove`, `Get`, `Merge` | [dict.md](std/dict.md) |
| [`Std.Result`](std/result.md) | Result helpers | `Unwrap`, `UnwrapOr`, `IsOk`, `IsError`, `Map`, `AndThen`, `OrElse` | [result.md](std/result.md) |
| [`Std.Option`](std/option.md) | Option helpers | `Unwrap`, `UnwrapOr`, `IsSome`, `IsNone`, `Map`, `AndThen`, `OrElse` | [option.md](std/option.md) |

---

## Quick examples

### Console I/O

```pascal
uses Std.Console;

WriteLn('Hello!');
WriteLn('Value: ', 42, ' Flag: ', true);
```

### Strings and conversions

```pascal
uses Std.Str, Std.Conv;

var Upper: string := ToUpper('hello');
var N: integer := StrToInt('42');
```

### Math

```pascal
uses Std.Math;

var Root: real := Sqrt(16.0);
var A: integer := Abs(-5);
```

### Arrays

```pascal
uses Std.Array;

mutable var Nums: array of integer := [3, 1, 4];
Push(Nums, 9);
var Sorted: array of integer := Sort(Nums);
```

### Error handling (Result and Option)

See [07-error-handling.md](07-error-handling.md) for the `Result of T, E` and `Option of T` types, constructors (`Ok`, `Error`, `Some`, `None`), the `try` operator, and `case` destructuring.

```pascal
uses Std.Result, Std.Option;

var R: Result of integer, string := Ok(42);
WriteLn(Std.Result.Unwrap(R));

var O: Option of integer := Some(7);
WriteLn(Std.Option.UnwrapOr(O, -1));
```
