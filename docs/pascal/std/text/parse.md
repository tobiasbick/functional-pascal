# `Std.Parse`

Structured parsing helpers for text input. `Std.Parse` is for callers that want a `Result` instead of the runtime errors raised by direct conversion routines in `Std.Conv`.

```pascal
program Example;
uses Std.Console, Std.Parse, Std.Result;
begin
  var Parsed: Result of integer, string := TryInt('42');
  WriteLn(UnwrapOr(Parsed, 0))
end.
```


## Importing and names

After `uses Std.Parse;` use short names (`TryInt`, `TryReal`, `TryBool`) or qualified names (`Std.Parse.TryInt`, etc.).

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
var R: Result of integer, string := TryInt(' +1_024 ');
WriteLn(UnwrapOr(R, 0))                       // 1024
```

---

## `function TryReal(Text: string): Result of real, string`

Parses Pascal real text. The text must include a fractional part; `1.0`, `-2.5`, and `1_024.0e-2` are valid, while `1e3`, `5.`, `NaN`, and `inf` are not.

```pascal
var R: Result of real, string := TryReal('1_024.0e-2');
WriteLn(UnwrapOr(R, 0.0))                     // 10.24
```

---

## `function TryBool(Text: string): Result of boolean, string`

Parses boolean text. Leading and trailing whitespace is ignored; casing does not matter.

```pascal
var R: Result of boolean, string := TryBool(' FALSE ');
WriteLn(UnwrapOr(R, true))                    // false
```

---

## Error handling

`Try*` functions do not raise runtime parse errors. Inspect the result with `Std.Result.IsOk` / `Std.Result.IsError`, recover with `Std.Result.UnwrapOr`, or destructure the result with `case`.

```pascal
case TryInt(Input) of
  Ok(N): WriteLn(N);
  Error(Message): WriteLn(Message)
end
```

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Registration | [`std_registry/loaded/parse.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/parse.rs) |
| Runtime | [`parse.rs`](../../../../crates/fpas-std/src/parse.rs) |
| Compiler | [`std_calls/parse.rs`](../../../../crates/fpas-compiler/src/compiler/std_calls/parse.rs) |
| Shared text | [`intrinsics.rs`](../../../../crates/fpas-std/src/intrinsics.rs) |
| Intrinsics | [`intrinsic/parse.rs`](../../../../crates/fpas-bytecode/src/intrinsic/parse.rs) |

## See also

- [Text and parsing index](README.md)
- [Standard library index](../README.md)
