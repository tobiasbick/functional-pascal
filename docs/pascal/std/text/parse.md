# `Std.Parse`

Structured parsing helpers for text input. `Std.Parse` is for callers that want a `Result` instead of the runtime errors raised by direct conversion routines in `Std.Conv`.

```pascal
program Example;

uses Std.Console as Console;
uses Std.Parse as Parse;
uses Std.Results as Results;

begin
  var Parsed: result of integer, string := Parse.TryInt('42');
  Console.WriteLn(Results.UnwrapOr(Parsed, 0));
end program;
```


## Importing and names

Import with `uses Std.Parse as Parse;`. Access every exported member through `Parse`, for example `Parse.TryInt(...)`. Imports open no short names.

---

## Quick reference

| Kind | Name | Notes |
|------|------|-------|
| function | `TryInt(Text: string): Result of integer, string` | trims whitespace; accepts Pascal integer text with `_` digit separators |
| function | `TryReal(Text: string): Result of real, string` | trims whitespace; requires a decimal point, for example `3.14` or `1.0e3` |
| function | `TryBool(Text: string): Result of boolean, string` | trims whitespace; accepts `true` and `false` case-insensitively |

---

## `function TryInt(Text: string): Result of integer, string`

Parses Pascal integer text. Returns `Ok(Value)` on success or `Error(Message)` on invalid text or overflow.

```pascal
uses Std.Console as Console;
uses Std.Parse as Parse;

var R: result of integer, string := Parse.TryInt(' +1_024 ');
Console.WriteLn(UnwrapOr(R, 0)); // 1024
```

---

## `function TryReal(Text: string): Result of real, string`

Parses Pascal real text. The text must include a fractional part; `1.0`, `-2.5`, and `1_024.0e-2` are valid, while `1e3`, `5.`, `NaN`, and `inf` are not.

```pascal
uses Std.Console as Console;
uses Std.Parse as Parse;

var R: result of real, string := Parse.TryReal('1_024.0e-2');
Console.WriteLn(UnwrapOr(R, 0.0)); // 10.24
```

---

## `function TryBool(Text: string): Result of boolean, string`

Parses boolean text. Leading and trailing whitespace is ignored; casing does not matter.

```pascal
uses Std.Console as Console;
uses Std.Parse as Parse;

var R: result of boolean, string := Parse.TryBool(' FALSE ');
Console.WriteLn(UnwrapOr(R, true)); // false
```

---

## Error handling

`Try*` functions do not raise runtime parse errors. Inspect the result with `Std.Results.IsOk` / `Std.Results.IsError`, recover with `Std.Results.UnwrapOr`, or destructure the result with `case`.

```pascal
uses Std.Console as Console;
uses Std.Parse as Parse;

case Parse.TryInt(Input) of
  when Ok(N):
    Console.WriteLn(N);
  when Error(Message):
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
