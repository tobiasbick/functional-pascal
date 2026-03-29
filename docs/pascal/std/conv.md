# `Std.Conv`

Explicit conversions between text and numbers (and simple numeric widens). Use this when you need **parsing** or **formatted text**, not silent coercion.

```pascal
program Example;
uses Std.Console, Std.Conv;
begin
  WriteLn(IntToStr(42))
end.
```

**Maintenance (implementers only):** align with [`std_registry/`](../../../crates/fpas-sema/src/std_registry/mod.rs), [`conv.rs`](../../../crates/fpas-std/src/conv.rs), [`intrinsics.rs`](../../../crates/fpas-std/src/intrinsics.rs), [`intrinsic.rs`](../../../crates/fpas-bytecode/src/intrinsic.rs), and [`compiler.rs`](../../../crates/fpas-compiler/src/compiler.rs).

---

## Importing and names

After `uses Std.Conv;` you may write **`IntToStr`**, **`StrToInt`**, … or **`Std.Conv.IntToStr`**, etc.

---

## Quick reference

Requires `uses Std.Conv;`.

| Kind | Name | Notes |
|------|------|--------|
| function | `IntToStr(N: integer): string` | decimal text |
| function | `StrToInt(S: string): integer` | parse; error if invalid |
| function | `IntToReal(N: integer): real` | widen |
| function | `RealToStr(R: real): string` | text form |
| function | `StrToReal(S: string): real` | parse; error if invalid |
| function | `CharToStr(C: char): string` | length-1 string |
| function | `BoolToStr(B: boolean): string` | `'true'` or `'false'` |
| function | `StrToBool(S: string): boolean` | parse; case-insensitive; error if invalid |
| function | `IntToHex(N: integer; Digits: integer): string` | uppercase hex, zero-padded |
| function | `HexToInt(S: string): integer` | parse hex; supports `$` / `0x` prefix |

---

## `function IntToStr(N: integer): string`

Decimal string representation of `N`.

```pascal
WriteLn(IntToStr(42))
```

---

## `function StrToInt(S: string): integer`

Parses an integer. Surrounding **whitespace is ignored**. **Runtime error** if the text is not a valid integer.

```pascal
WriteLn(StrToInt('  -7  '))
```

---

## `function IntToReal(N: integer): real`

Converts integer to `real` (exact for integers in the representable range).

```pascal
var X: real := IntToReal(3);
WriteLn(X)
```

---

## `function RealToStr(R: real): string`

Returns a string representation of `R` (format follows the runtime).

```pascal
WriteLn(RealToStr(1.5))
```

---

## `function StrToReal(S: string): real`

Parses a floating-point value. Surrounding **whitespace is ignored**. **Runtime error** if invalid.

```pascal
WriteLn(StrToReal('2.25'))
```

---

## `function CharToStr(C: char): string`

Single-character string.

```pascal
WriteLn(CharToStr('Z'))
```

---

## `function BoolToStr(B: boolean): string`

Returns `'true'` or `'false'`.

```pascal
WriteLn(BoolToStr(true))   { true }
WriteLn(BoolToStr(false))  { false }
```

---

## `function StrToBool(S: string): boolean`

Parses `'true'` or `'false'` (case-insensitive). **Runtime error** if `S` is neither.

```pascal
WriteLn(StrToBool('True'));    { true }
WriteLn(StrToBool('FALSE'))    { false }
```

---

## `function IntToHex(N: integer; Digits: integer): string`

Returns `N` as an uppercase hexadecimal string, zero-padded to at least `Digits` characters.

```pascal
WriteLn(IntToHex(255, 2));    { FF }
WriteLn(IntToHex(255, 4))     { 00FF }
```

---

## `function HexToInt(S: string): integer`

Parses a hexadecimal string. Accepts optional `$` or `0x` prefix. **Runtime error** if the string is not valid hex.

```pascal
WriteLn(HexToInt('FF'));     { 255 }
WriteLn(HexToInt('$FF'));    { 255 }
WriteLn(HexToInt('0xFF'))    { 255 }
```

---

## Implementation map (contributors)

| Concern | Location |
|---------|-----------|
| Implementations | [`conv.rs`](../../../crates/fpas-std/src/conv.rs) |
| Registration | [`std_registry.rs`](../../../crates/fpas-sema/src/std_registry.rs) |

[← Standard library index](README.md)
