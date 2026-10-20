# `Std.Parse`

Text parsers returning `Result of (T, Parse.ParseError)`. Use `case` to handle input
locally, `try` to propagate a returned error, or `.Unwrap()` for deliberate abort.
Formatting and conversions of typed values belong to [Std.Conv](conv.md).

```pascal id=example-1
program Example;

uses Std.Console as Console, Std.Parse as Parse;

begin
  const Parsed: result of (integer, Parse.ParseError) := Parse.Int('42');
  case Parsed of
    when Ok(const Value):
      Console.WriteLn(Value);
    when Error(const Message):
      Console.WriteLn(Message);
  end case;
end.
```

## Importing and names

Import with `uses Std.Parse as Parse;` and access public symbols as `Parse.Name`. User enum variants require `Parse.Type.Variant`.

---

## Quick reference

| Kind | Name | Notes |
|------|------|-------|
| function | `Int(Text: string): Result of (integer, Parse.ParseError)` | signed decimal or `$` hex with `_` separators; trims whitespace and checks signed 64-bit bounds |
| function | `Real(Text: string): Result of (real, Parse.ParseError)` | finite fractional/exponent text; preserves signed zero and rejects nonzero underflow to zero |
| function | `Bool(Text: string): Result of (boolean, Parse.ParseError)` | trims whitespace; accepts `true` and `false` case-insensitively |
| function | `HexInt(Text: string): Result of (integer, Parse.ParseError)` | signed hexadecimal text; bare hex or `$`/`0x`/`0X` prefix; checked signed 64-bit bounds |

---

## `function Int(Text: string): Result of (integer, Parse.ParseError)`

Parses signed decimal or `$` hexadecimal integer text with single `_` separators.
Surrounding whitespace is ignored. The complete signed 64-bit range is accepted,
including `-9223372036854775808` and `-$8000000000000000`. Hex is a mathematical
magnitude, not an unsigned bit-pattern reinterpretation. Returns `Ok(Value)` on
success or `Error(Cause)` on invalid text or overflow. A deliberate abort is
`Parse.Int(Text).Unwrap()`.

```pascal id=example-2 fragment context=body-console-parse
const R: result of (integer, Parse.ParseError) := Parse.Int(' +1_024 ');
case R of
  when Ok(const Value):
    Console.WriteLn(Value); // 1024
  when Error(const Message):
    Console.WriteLn(Message);
end case;
```

---

## `function Real(Text: string): Result of (real, Parse.ParseError)`

Parses Pascal real text with an optional sign and surrounding whitespace. The
text must include a fractional part; `1.0`, `-2.5`, and `1_024.0e-2` are valid,
while `1e3`, `5.`, `NaN`, and `inf` are not. Overflow and nonzero input rounded
to zero return Error; representable subnormals remain valid. Negative zero is
preserved. Invalid text returns Error; `Parse.Real(Text).Unwrap()` aborts deliberately.

For every finite R, `Real(RealToStr(R))` returns the identical binary64 bits,
including integral reals and either sign of zero. See
[RealToStr](conv.md#function-realtostrr-real-string).

```pascal id=example-3 fragment context=body-console-parse
const R: result of (real, Parse.ParseError) := Parse.Real('1_024.0e-2');
case R of
  when Ok(const Value):
    Console.WriteLn(Value); // 10.24
  when Error(const Message):
    Console.WriteLn(Message);
end case;
```

---

## `function Bool(Text: string): Result of (boolean, Parse.ParseError)`

Parses boolean text. Leading and trailing whitespace is ignored; casing does not matter.

```pascal id=example-4 fragment context=body-console-parse
const R: result of (boolean, Parse.ParseError) := Parse.Bool(' FALSE ');
case R of
  when Ok(const Value):
    Console.WriteLn(Value); // false
  when Error(const Message):
    Console.WriteLn(Message);
end case;
```

---

## `function HexInt(Text: string): Result of (integer, Parse.ParseError)`

Parses signed hexadecimal text with single `_` separators and an optional `$`,
`0x`, or `0X` prefix. Surrounding whitespace is ignored; the sign precedes the
prefix. Bare hex such as `FF` is accepted. Hex is a mathematical magnitude, not
an unsigned bit-pattern reinterpretation. The complete signed 64-bit range is
accepted; invalid text and overflow return Error(Cause).

HexInt and Int have distinct interpretations: `Parse.HexInt('10')` returns
`Ok(16)`, whereas `Parse.Int('10')` returns `Ok(10)`. General Int requires `$`
for hex and rejects the `0x`/`0X` prefix.

```pascal id=example-5 fragment context=body-console-parse
const R := Parse.HexInt('-0x8000000000000000');
case R of
  when Ok(const Value):
    Console.WriteLn(Value); // -9223372036854775808
  when Error(const Message):
    Console.WriteLn(Message);
end case;
```

## Error handling

Invalid text is a returned Error. Test its variant without binding data using
`Value is Ok(_)` or `Value is Error(_)`. Use case to bind a payload or handle an
error locally, and try to propagate an error from a compatible Result-returning
function. Unwrap is the explicit runtime-failure boundary; try does not catch it.

```pascal id=example-6 fragment context=parse-input
case Parse.Int(Input) of
  when Ok(const N):
    Console.WriteLn(N);
  when Error(const Message):
    Console.WriteLn(Message);
end case;
```

A plain default belongs in a case expression's failure arm, which runs only
when selected:

```pascal id=example-7 fragment context=parse-input
const Count: integer := case Parse.Int(Input) of
  when Ok(const Value): Value;
  when Error(_): 0;
end case;
```

All four parsers have the named parameter Text, accept positional or fully named
calls, and can be used as ordinary callable values with matching signatures:

```pascal id=example-8 fragment context=body-parse
const Read: function(Text: string): result of (integer, Parse.ParseError) := Parse.Int;
const Value := Read(Text := '42');
```

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Registration | [`std_registry/loaded/parse.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/parse.rs) |
| Runtime | [`parse.rs`](../../../../crates/fpas-std/src/parse.rs) |
| Compiler intrinsic catalog | [`intrinsic_catalog.rs`](../../../../crates/fpas-compiler/src/intrinsic_catalog.rs) |
| Shared text | [`intrinsics.rs`](../../../../crates/fpas-std/src/intrinsics.rs) |
| Intrinsics | [`intrinsic/parse.rs`](../../../../crates/fpas-bytecode/src/intrinsic/parse.rs) |

## See also

- [Text and parsing index](README.md)
- [Standard library index](../README.md)

## Typed failures

`ParseError.InvalidSyntax(Target, Detail)` reports malformed complete input; `ParseError.OutOfRange(Target, Detail)` reports valid numeric syntax outside the signed 64-bit or finite binary64 range, including nonzero underflow to zero. Syntax is checked before range: an overflowing prefix with invalid trailing text is InvalidSyntax. Target is `integer`, `hexadecimal integer`, `real`, or `boolean`. The accepted grammar is unchanged.
