# `Std.Conv`

Formatting of typed values and explicit integer-to-real conversion. Parse input
text with [Std.Parse](parse.md), whose operations return Result.

```pascal
program Example;

uses Std.Console as Console, Std.Conv as Conv;

begin
  Console.WriteLn(Conv.IntToStr(42));
end.
```

## Importing and names

After `uses Std.Conv as Conv;`, call `Conv.IntToStr`, `Conv.IntToReal`, and the
other exported routines through that alias.

---

## Quick reference

Requires `uses Std.Conv as Conv;`.

| Kind | Name | Notes |
|------|------|--------|
| function | `IntToStr(N: integer): string` | decimal text |
| function | `IntToReal(N: integer): real` | explicit binary64 rounding; may lose integer precision |
| function | `RealToStr(R: real): string` | finite real text with a bit-preserving parsing round-trip; integral reals include `.0` |
| function | `BoolToStr(B: boolean): string` | `'true'` or `'false'` |
| function | `IntToHex(N: integer; Digits: integer): string` | uppercase hex, zero-padded |

---

## `function IntToStr(N: integer): string`

Decimal string representation of `N`.

```pascal
Console.WriteLn(Conv.IntToStr(42));
```

---

## `function IntToReal(N: integer): real`

Converts integer to binary64 using rounding to nearest, with ties to even.
All integer values fit the real value range, but conversion may lose precision.
Every integer between `-9007199254740992` and `9007199254740992` is exact;
larger values are exact only when representable at binary64's spacing.
This is the explicit conversion used before mixing integer and real calculations.

```pascal
const X: real := Conv.IntToReal(3);
Console.WriteLn(X);
```

---

## `function RealToStr(R: real): string`

Returns locale-independent real text that `Parse.Real` parses back
to the identical binary64 bits. Integral reals include `.0`, so `RealToStr(1.0)`
is `'1.0'`. Negative zero is `'-0.0'`. Nonfinite values are rejected.

```pascal
Console.WriteLn(Conv.RealToStr(1.5));
```

---

## `function BoolToStr(B: boolean): string`

Returns `'true'` or `'false'`.

```pascal
Console.WriteLn(Conv.BoolToStr(true));   // true
Console.WriteLn(Conv.BoolToStr(false));  // false
```

---

## `function IntToHex(N: integer; Digits: integer): string`

Returns `N` as an uppercase hexadecimal string, zero-padded to at least `Digits` characters.

`Digits` must be at most **1_000_000**. Larger values raise a runtime error.

```pascal
Console.WriteLn(Conv.IntToHex(255, 2));    // FF
Console.WriteLn(Conv.IntToHex(255, 4));     // 00FF
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
