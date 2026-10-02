# `Std.Conv`

Explicit conversions between text and numbers (and simple numeric widens). Use this when you need **parsing** or **formatted text**, not silent coercion.

```pascal
program Example;

uses Std.Console as Console;
uses Std.Conv as Conv;

begin
  Console.WriteLn(Conv.IntToStr(42));
end program;
```


## Importing and names

Import with `uses Std.Conv as Conv;`. Access every exported member through `Conv`, for example `Conv.IntToStr(...)`. Imports open no short names.

---

## Quick reference

Requires `uses Std.Conv as Conv;`.

| Kind | Name | Notes |
|------|------|--------|
| function | `IntToStr(N: integer): string` | decimal text |
| function | `StrToInt(S: string): integer` | parse; error if invalid |
| function | `IntToReal(N: integer): real` | widen |
| function | `RealToStr(R: real): string` | text form |
| function | `StrToReal(S: string): real` | parse; error if invalid |
| function | `BoolToStr(B: boolean): string` | `'true'` or `'false'` |
| function | `StrToBool(S: string): boolean` | parse; case-insensitive; error if invalid |
| function | `IntToHex(N: integer; Digits: integer): string` | uppercase hex, zero-padded |
| function | `HexToInt(S: string): integer` | parse hex; supports `$` / `0x` prefix |

---

## `function IntToStr(N: integer): string`

Decimal string representation of `N`.

```pascal
uses Std.Console as Console;
uses Std.Conv as Conv;

Console.WriteLn(Conv.IntToStr(42));
```

---

## `function StrToInt(S: string): integer`

Parses an integer. Surrounding **whitespace is ignored**. **Runtime error** if the text is not a valid integer.

```pascal
uses Std.Console as Console;
uses Std.Conv as Conv;

Console.WriteLn(Conv.StrToInt('  -7  '));
```

---

## `function IntToReal(N: integer): real`

Converts integer to `real` (exact for integers in the representable range).

```pascal
uses Std.Console as Console;
uses Std.Conv as Conv;

var X: real := Conv.IntToReal(3);
Console.WriteLn(X);
```

---

## `function RealToStr(R: real): string`

Returns a string representation of `R` (format follows the runtime).

```pascal
uses Std.Console as Console;
uses Std.Conv as Conv;

Console.WriteLn(Conv.RealToStr(1.5));
```

---

## `function StrToReal(S: string): real`

Parses a floating-point value. Surrounding **whitespace is ignored**. **Runtime error** if invalid.

```pascal
uses Std.Console as Console;
uses Std.Conv as Conv;

Console.WriteLn(Conv.StrToReal('2.25'));
```

---

## `function BoolToStr(B: boolean): string`

Returns `'true'` or `'false'`.

```pascal
uses Std.Console as Console;
uses Std.Conv as Conv;

Console.WriteLn(Conv.BoolToStr(true));   // true
Console.WriteLn(Conv.BoolToStr(false));  // false
```

---

## `function StrToBool(S: string): boolean`

Parses `'true'` or `'false'` (case-insensitive). **Runtime error** if `S` is neither.

```pascal
uses Std.Console as Console;
uses Std.Conv as Conv;

Console.WriteLn(Conv.StrToBool('True')); // true
Console.WriteLn(Conv.StrToBool('FALSE')); // false
```

---

## `function IntToHex(N: integer; Digits: integer): string`

Returns `N` as an uppercase hexadecimal string, zero-padded to at least `Digits` characters.

`Digits` must be at most **1_000_000**. Larger values raise a runtime error.

```pascal
uses Std.Console as Console;
uses Std.Conv as Conv;

Console.WriteLn(Conv.IntToHex(255, 2)); // FF
Console.WriteLn(Conv.IntToHex(255, 4)); // 00FF
```

---

## `function HexToInt(S: string): integer`

Parses a hexadecimal string. Accepts optional `$` or `0x` prefix. **Runtime error** if the string is not valid hex.

```pascal
uses Std.Console as Console;
uses Std.Conv as Conv;

Console.WriteLn(Conv.HexToInt('FF')); // 255
Console.WriteLn(Conv.HexToInt('$FF')); // 255
Console.WriteLn(Conv.HexToInt('0xFF')); // 255
```

---

## Implementation (contributors)

| Concern | Location |
|---------|-----------|
| Implementations | [`conv.rs`](../../../../crates/fpas-std/src/conv.rs) |
| Registration | [`std_registry/mod.rs`](../../../../crates/fpas-sema/src/std_registry/mod.rs) |

## See also

- [Text and parsing index](README.md)
- [Standard library index](../README.md)
