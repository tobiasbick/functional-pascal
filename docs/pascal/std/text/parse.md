# `Std.Parse`

Structured parsing helpers for text input. `Std.Parse` is for callers that want a `Result` instead of the runtime errors raised by direct conversion routines in `Std.Conv`.

```pascal
program Example;

uses Std.Console as Console, Std.Parse as Parse;

begin
  const Parsed: result of (integer, string) := Parse.TryInt('42');
  Console.WriteLn(Parsed.UnwrapOr(0));
end.
```


## Importing and names

Import with `uses Std.Parse as Parse;` and access public symbols as `Parse.Name`. User enum variants require `Parse.Type.Variant`.

---

## Quick reference

| Kind | Name | Notes |
|------|------|-------|
| function | `TryInt(Text: string): Result of (integer, string)` | signed decimal or `$` hex with `_` separators; trims whitespace and checks signed 64-bit bounds |
| function | `TryReal(Text: string): Result of (real, string)` | finite fractional/exponent text; preserves signed zero and rejects nonzero underflow to zero |
| function | `TryBool(Text: string): Result of (boolean, string)` | trims whitespace; accepts `true` and `false` case-insensitively |

---

## `function TryInt(Text: string): Result of (integer, string)`

Parses signed decimal or `$` hexadecimal integer text with single `_` separators.
Surrounding whitespace is ignored. The complete signed 64-bit range is accepted,
including `-9223372036854775808` and `-$8000000000000000`. Hex is a mathematical
magnitude, not an unsigned bit-pattern reinterpretation. Returns `Ok(Value)` on
success or `Error(Message)` on invalid text or overflow. `StrToInt` uses the same
grammar with a runtime error instead of Result.

```pascal
const R: Result of (integer, string) := Parse.TryInt(' +1_024 ');
Console.WriteLn(R.UnwrapOr(0));                       // 1024
```

---

## `function TryReal(Text: string): Result of (real, string)`

Parses Pascal real text with an optional sign and surrounding whitespace. The
text must include a fractional part; `1.0`, `-2.5`, and `1_024.0e-2` are valid,
while `1e3`, `5.`, `NaN`, and `inf` are not. Overflow and nonzero input rounded
to zero return Error; representable subnormals remain valid. Negative zero is
preserved. `StrToReal` uses the same grammar with a runtime error instead of Result.

For every finite R, `TryReal(RealToStr(R))` returns the identical binary64 bits,
including integral reals and either sign of zero. See
[RealToStr](conv.md#function-realtostrr-real-string).

```pascal
const R: Result of (real, string) := Parse.TryReal('1_024.0e-2');
Console.WriteLn(R.UnwrapOr(0.0));                     // 10.24
```

---

## `function TryBool(Text: string): Result of (boolean, string)`

Parses boolean text. Leading and trailing whitespace is ignored; casing does not matter.

```pascal
const R: Result of (boolean, string) := Parse.TryBool(' FALSE ');
Console.WriteLn(R.UnwrapOr(true));                    // false
```

---

## Error handling

`Try*` functions do not raise runtime parse errors. Inspect the result with `.IsOk()` / `.IsError()`, recover with `.UnwrapOr(Default)`, or destructure the result with `case`.

```pascal
case Parse.TryInt(Input) of
  when Ok(const N):
    Console.WriteLn(N);
  when Error(const Message):
    Console.WriteLn(Message);
end case;
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
