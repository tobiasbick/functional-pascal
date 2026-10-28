# `Std.Text.Format`

Locale-independent text formatting of typed values. Parse input text with
[Std.Text.Parse](parse.md), whose operations return Result. Numeric conversion
belongs to [Std.Math.IntToReal](../numeric/math.md#function-inttorealn-integer-real).

```pascal id=example-1
program Example;

uses Std.Console as Console, Std.Text.Format as Format;

begin
  Console.WriteLn(Format.IntToStr(42));
end.
```

## Importing and names

After `uses Std.Text.Format as Format;`, call `Format.IntToStr` and the other
exported routines through that alias.

---

## Quick reference

Requires `uses Std.Text.Format as Format;`.

| Kind | Name | Notes |
|------|------|--------|
| function | `IntToStr(N: integer): string` | decimal text |
| function | `RealToStr(R: real): string` | finite real text with a bit-preserving parsing round-trip; integral reals include `.0` |
| function | `BoolToStr(B: boolean): string` | `'true'` or `'false'` |
| function | `IntToHex(N: integer; Digits: integer): string` | uppercase hex, zero-padded |

---

## `function IntToStr(N: integer): string`

Decimal string representation of `N`.

```pascal id=example-2 fragment context=body-console-format
Console.WriteLn(Format.IntToStr(42));
```

---

## `function RealToStr(R: real): string`

Returns locale-independent real text that `Parse.Real` parses back
to the identical binary64 bits. Integral reals include `.0`, so `RealToStr(1.0)`
is `'1.0'`. Negative zero is `'-0.0'`. Nonfinite values are rejected.

```pascal id=example-4 fragment context=body-console-format
Console.WriteLn(Format.RealToStr(1.5));
```

---

## `function BoolToStr(B: boolean): string`

Returns `'true'` or `'false'`.

```pascal id=example-5 fragment context=body-console-format
Console.WriteLn(Format.BoolToStr(true)); // true
Console.WriteLn(Format.BoolToStr(false)); // false
```

---

## `function IntToHex(N: integer; Digits: integer): string`

Returns `N` as an uppercase hexadecimal string, zero-padded to at least `Digits` characters.

`Digits` must be at most **1_000_000**. Larger values raise a runtime error.

```pascal id=example-6 fragment context=body-console-format
Console.WriteLn(Format.IntToHex(255, 2)); // FF
Console.WriteLn(Format.IntToHex(255, 4)); // 00FF
```

---

## Implementation (contributors)

| Concern | Location |
|---------|-----------|
| Implementations | [`conv/mod.rs`](../../../../crates/fpas-runtime/src/stdlib/conv/mod.rs), shared [`numeric text`](../../../../crates/fpas-numeric/src/text.rs) |
| Registration | [`std_registry/mod.rs`](../../../../crates/fpas-sema/src/std_registry/mod.rs) |

## See also

- [Text and parsing index](README.md)
- [Standard library index](../README.md)
