# `Std.Conv`

Explicit conversions between text and numbers (and simple numeric widens). Use this when you need **parsing** or **formatted text**, not silent coercion.

```pascal
program Example;
uses Std.Console, Std.Conv;
begin
  WriteLn(IntToStr(42));
end.
```


## Importing and names

After `uses Std.Conv;` you may write **`IntToStr`**, **`StrToInt`**, … or **`Std.Conv.IntToStr`**, etc.

---

## Quick reference

Requires `uses Std.Conv;`.

| Kind | Name | Notes |
|------|------|--------|
| function | `IntToStr(N: integer): string` | decimal text |
| function | `StrToInt(S: string): integer` | signed decimal or `$` hex text; checked signed 64-bit range |
| function | `IntToReal(N: integer): real` | explicit binary64 rounding; may lose integer precision |
| function | `RealToStr(R: real): string` | finite real text with a bit-preserving parsing round-trip; integral reals include `.0` |
| function | `StrToReal(S: string): real` | finite real literal text; rejects nonzero underflow to zero |
| function | `BoolToStr(B: boolean): string` | `'true'` or `'false'` |
| function | `StrToBool(S: string): boolean` | parse; case-insensitive; error if invalid |
| function | `IntToHex(N: integer; Digits: integer): string` | uppercase hex, zero-padded |
| function | `HexToInt(S: string): integer` | parse hex; supports `$` / `0x` prefix |

---

## `function IntToStr(N: integer): string`

Decimal string representation of `N`.

```pascal
WriteLn(IntToStr(42));
```

---

## `function StrToInt(S: string): integer`

Parses signed decimal or `$` hexadecimal integer text with single `_` separators
between digits. Surrounding whitespace is ignored. The result must fit the signed
64-bit range, including `-9223372036854775808` and `-$8000000000000000`.
Invalid text or overflow is a runtime error. `Std.Parse.TryInt` uses the same
grammar and returns a Result.

```pascal
WriteLn(StrToInt('  -7  '));
```

---

## `function IntToReal(N: integer): real`

Converts integer to binary64 using rounding to nearest, with ties to even.
All integer values fit the real value range, but conversion may lose precision.
Every integer between `-9007199254740992` and `9007199254740992` is exact;
larger values are exact only when representable at binary64's spacing.
This is the explicit conversion used before mixing integer and real calculations.

```pascal
const X: real := IntToReal(3);
WriteLn(X);
```

---

## `function RealToStr(R: real): string`

Returns locale-independent real text that `StrToReal` and `TryReal` parse back
to the identical binary64 bits. Integral reals include `.0`, so `RealToStr(1.0)`
is `'1.0'`. Negative zero is `'-0.0'`. Nonfinite values are rejected.

```pascal
WriteLn(RealToStr(1.5));
```

---

## `function StrToReal(S: string): real`

Parses the real literal forms: digits on both sides of `.`, an optional exponent,
and single `_` digit separators. A leading sign and surrounding whitespace are
accepted. NaN, infinity, overflow, and nonzero input rounded to zero are invalid;
representable subnormals remain valid. Invalid input is a runtime error.
`Std.Parse.TryReal` uses the same grammar and returns a Result.

```pascal
WriteLn(StrToReal('2.25'));
```

---

## `function BoolToStr(B: boolean): string`

Returns `'true'` or `'false'`.

```pascal
WriteLn(BoolToStr(true));   // true
WriteLn(BoolToStr(false));  // false
```

---

## `function StrToBool(S: string): boolean`

Parses `'true'` or `'false'` (case-insensitive). **Runtime error** if `S` is neither.

```pascal
WriteLn(StrToBool('True'));    // true
WriteLn(StrToBool('FALSE'));    // false
```

---

## `function IntToHex(N: integer; Digits: integer): string`

Returns `N` as an uppercase hexadecimal string, zero-padded to at least `Digits` characters.

`Digits` must be at most **1_000_000**. Larger values raise a runtime error.

```pascal
WriteLn(IntToHex(255, 2));    // FF
WriteLn(IntToHex(255, 4));     // 00FF
```

---

## `function HexToInt(S: string): integer`

Parses signed hexadecimal text with single `_` separators and an optional `$`,
`0x`, or `0X` prefix. The sign precedes the prefix. Bare hex such as `'FF'`
is accepted by this explicit hex helper; general integer parsers require `$`
for hex. Values must fit the signed 64-bit range; invalid text is a runtime error.

```pascal
WriteLn(HexToInt('FF'));     // 255
WriteLn(HexToInt('$FF'));    // 255
WriteLn(HexToInt('0xFF'));    // 255
```

---

## Implementation (contributors)

| Concern | Location |
|---------|-----------|
| Implementations | [`conv/mod.rs`](../../../../crates/fpas-std/src/conv/mod.rs), shared [`numeric text`](../../../../crates/fpas-numeric/src/text.rs) |
| Registration | [`std_registry/mod.rs`](../../../../crates/fpas-sema/src/std_registry/mod.rs) |

## See also

- [Text and parsing index](README.md)
- [Standard library index](../README.md)
