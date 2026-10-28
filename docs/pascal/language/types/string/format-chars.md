# Format and characters

## `Text.IsNumeric(): boolean`

`true` if the string (after trim) parses as an **integer** or **real**, otherwise `false`.
It uses the same bounded grammar as `Std.Text.Parse.Int` and `Real`, including
signed `$` hex, digit separators, and finite real values. Nonzero real input
that rounds to zero is not numeric. See [numeric parsing](../../../std/text/parse.md).

```pascal id=example-1 fragment context=body-console
Console.WriteLn('42'.IsNumeric());
Console.WriteLn('nope'.IsNumeric());
```

---

## Scalar indexing: `Text[Index]`

Returns the character at the 0-based `Index`. **Runtime error** if out of bounds.

Indices count Unicode scalars. For a nonempty string, valid indices are
`0..S.Length()-1`. Empty strings have no valid character index. Invalid indices
raise a runtime error. Check `S.IsEmpty()` before reading an empty string.

```pascal id=example-2 fragment context=body-console
const C: string := 'Hello'[0];
Console.WriteLn(C); // H
```

---

## `Text.SetCharAt(Index: integer; C: string): string`

Returns a **new** string that is identical to `S` except the character at `Index` is replaced with `C`. `C` must contain exactly one Unicode scalar value. **Runtime error** if `Index` is out of bounds or `C` is empty or contains multiple characters.

`Index` follows the same scalar bounds and empty-string checks as scalar indexing.

```pascal id=example-3 fragment context=body-console
Console.WriteLn('Hello'.SetCharAt(0, 'J')); // Jello
```

---

## `Text.Ord(): integer`

Returns the Unicode codepoint (integer value) of the receiver. The receiver must contain exactly one Unicode scalar value; empty or multi-character strings are runtime errors.

```pascal id=example-4 fragment context=body-console
Console.WriteLn('A'.Ord()); // 65
```

---

## `string.Chr(N: integer): string`

Returns the character with Unicode scalar value `N`: `0..1114111`, excluding
`55296..57343`. **Runtime error** if `N` is outside this domain. For a valid
decimal value, the [`#N` literal](../../basics/primitive-types.md#character-codes)
produces the same one-scalar string at compile time; invalid literals are lexical
errors. Values such as 255 denote Unicode scalars rather than raw encoded bytes.

```pascal id=example-5 fragment context=body-console
Console.WriteLn(string.Chr(65)); // A
Console.WriteLn(string.Chr(8364) = #8_364); // true
Console.WriteLn(#128512.Length()); // 1 (😀)
```

---

## `Text.Format(Arguments...): string`

Returns a new string by substituting format specifiers in `Template` with the supplied arguments.

The string receiver is the template. Its heterogeneous arguments are positional only; named arguments are rejected.

```pascal id=example-6 fragment context=body
const Zoom: real := 2.0;
const CX: real := 0.5;
const CY: real := -0.5;
const Index: integer := 3;
const Name: string := 'sample';
const Status: string := 'Zoom: %fx Center: (%f, %f)'.Format(Zoom, CX, CY);
const Msg: string := 'Item %d: %s'.Format(Index, Name);
const Pct: string := '100%%'.Format(); // '100%'
```

### Specifiers

| Specifier | Accepted type | Example |
|-----------|--------------|---------|
| `%d` | `integer` | `'%d'.Format(42)` → `'42'` |
| `%f` | `real` or `integer` | `'%f'.Format(3.14)` → `'3.14'` |
| `%e` / `%E` | `real` or `integer` | `'%.2e'.Format(1250)` → `'1.25e+03'` |
| `%s` | `string` | `'%s'.Format('hi')` → `'hi'` |
| `%%` | *(no argument)* | `'100%%'.Format()` → `'100%'` |

Numeric conversions use `%[flags][width][.precision]kind`, where `kind` is
`d`, `f`, `e`, or `E`. Width and precision, when present, contain decimal digits:

| Field | Meaning | Example |
|-------|---------|---------|
| `width` | Minimum field width, padded with spaces; never truncates a value | `'%8d'.Format(42)` → `'      42'` |
| `0` flag | Pad with zeroes after a leading minus sign | `'%08d'.Format(-42)` → `'-0000042'` |
| `-` flag | Align left and pad with spaces on the right | `'%-8d'.Format(42)` → `'42      '` |
| `.precision` | Digits after the decimal point for `f`, `e`, and `E` | `'%10.2f'.Format(3.5)` → `'      3.50'` |

The `-` and `0` flags may appear in either order, once each. If both occur,
left alignment takes precedence over zero padding. Precision zero emits no
decimal point. Precision is not supported for `d`; `%s` and `%%` accept no
flags, width, or precision. Formats such as `%g`, `%G`, dynamic `*` widths,
and locale-specific separators are unsupported.

`f` without precision retains the existing formatting: it emits at least one
fractional digit, so `'%f'.Format(42)` produces `'42.0'`. Integer arguments are
converted to real in that form, including when a field width or flag is used.
With explicit precision, `f` formats integer arguments directly from their
exact decimal value.

`e` and `E` use scientific notation with one digit before the decimal point.
The default precision is six fractional digits. The exponent uses the chosen
`e` or `E`, an explicit sign, and at least two digits:
`'%E'.Format(42)` produces `'4.200000E+01'`. Integer arguments are formatted
from their exact decimal value without conversion to real.

Real arguments use their stored binary64 value. Fixed precision and scientific
notation round to the nearest value, with ties choosing the even final digit.
The sign of negative zero is preserved. Formatting uses `.` as the decimal
separator and does not depend on the host locale.

```pascal id=example-7 run
program NumericFormatting;

uses Std.Test as Test;

begin
  Test.AssertEquals('3.14', '%.2f'.Format(3.14159));
  Test.AssertEquals('      42', '%8d'.Format(42));
  Test.AssertEquals('-0000042', '%08d'.Format(-42));
  Test.AssertEquals('42      ', '%-08d'.Format(42));
  Test.AssertEquals('42      ', '%0-8d'.Format(42));
  Test.AssertEquals('      3.50', '%10.2f'.Format(3.5));
  Test.AssertEquals('1.25e+03', '%.2e'.Format(1250));
  Test.AssertEquals('4.200000E+01', '%E'.Format(42));
  Test.AssertEquals('2', '%.0f'.Format(2.5));
  Test.AssertEquals('4', '%.0f'.Format(3.5));
  Test.AssertEquals('-0.00', '%.2f'.Format(-0.0));
  Test.AssertEquals('9007199254740993.0', '%.1f'.Format(9_007_199_254_740_993));
  var Template: string := '%08d';
  Template.FormatInPlace(42);
  Test.AssertEquals('00000042', Template);
end.
```

### Limits

Width is at most **1_000_000** and precision at most **1024**. Within one call,
numeric conversions with a flag, width, or precision, and all `e`/`E`
conversions, may emit at most **1_000_000 UTF-8 bytes in total**, including
their padding. Bare `%d` and `%f`, `%s`, literal text, and `%%` do not count
toward that limit and retain their existing behavior.

### Runtime errors

Invalid templates raise `FP5022`, including a trailing `%`, unknown kinds,
repeated flags, missing precision digits, unsupported modifiers, or a width,
precision, or numeric-output limit violation. Too few or too many arguments
and argument types that do not match their conversion also raise `FP5022`.
`%s` requires a string; use an error value's `.Message()` explicitly when
formatting its message.

---

## See also

- [Str overview](README.md)
- [`Std.Text.Format`](../../../std/text/format.md)
- [Text index](../../../std/text/README.md)
