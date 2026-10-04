# `Std.Toml`

Parse and stringify TOML 1.0 documents with an explicit Functional Pascal value representation.

```pascal
program Example;

uses Std.Console as Console;
uses Std.Str as Str;
uses Std.Toml as Toml;

begin
  const Parsed: result of (Toml.TomlValue, string) := Toml.Parse(('[project]' + Str.Chr(10)) +
                                                             'name = ''demo''');
  case Parsed of
    when Result.Ok(const Value):
      Console.WriteLn(Toml.Stringify(Value));
    when Result.Error(const Message):
      Console.WriteLn(Message);
  end case;
end program;
```

## Importing and names

Import with `uses Std.Toml as Toml;`. Access every exported member through `Toml`, for example `Toml.Parse(...)`. Imports open no short names.

`TomlValue` is an enum. Tables use `dict of (string, TomlValue)`; arrays use `array of (TomlValue)`.

## Quick reference

| Kind | Name | Notes |
| --- | --- | --- |
| type | `TomlValue` | TOML value tree |
| function | `Parse(Text: string): Result of (TomlValue, string)` | Parses one TOML document |
| function | `Stringify(Value: TomlValue): string` | Encodes a TOML value tree |

### `TomlValue`

```pascal
type TomlValue = enum
  String(Value: string);
  Integer(Value: integer);
  Float(Value: real);
  Boolean(Value: boolean);
  Datetime(Value: string);
  ArrayValue(Items: array of (TomlValue));
  Table(Fields: dict of (string, TomlValue));
end enum;
```

`Datetime` preserves the TOML date, time, local date-time, or offset date-time spelling returned by the parser. Its `Value` must be a valid TOML date/time string when passed to `Stringify`.

## `Parse`

```pascal
function Parse(Text: string): Result of (TomlValue, string);
```

Parses a TOML document. Valid input returns `Result.Ok(TomlValue)`; syntax errors return `Result.Error(Message)` rather than aborting the program. TOML documents are tables at the root, so successful `Parse` results always have the `TomlValue.Table` variant.

All TOML 1.0 value kinds are represented: strings, signed 64-bit integers, floating-point values (including `inf` and `nan`), booleans, date/time values, arrays, tables, inline tables, and arrays of tables.

```pascal
uses Std.Console as Console;
uses Std.Toml as Toml;

const Parsed: result of (Toml.TomlValue, string) := Toml.Parse(
  'title = ''example''' + Chr(10) +
  'enabled = true' + Chr(10) +
  '[server]' + Chr(10) +
  'port = 8080'
);
case Parsed of
  when Result.Ok(const Value): Console.WriteLn('parsed');
  when Result.Error(const Message): Console.WriteLn('TOML error: ' + Message);
end case;
```

## `Stringify`

```pascal
function Stringify(Value: TomlValue): string;
```

Encodes a `TomlValue` tree as TOML. The supplied root must be a table because TOML documents have table roots. `Stringify` raises a runtime error for malformed manually constructed values, non-string table keys, invalid date/time text, or nesting deeper than 256 levels.

```pascal
uses Std.Console as Console;
uses Std.Toml as Toml;

const Value: Toml.TomlValue := Toml.TomlValue.Table(['project': Toml.TomlValue.Table(['name': Toml.TomlValue.String('demo'), 'version': Toml.TomlValue.Integer(1)])]);
Console.WriteLn(Toml.Stringify(Value));
```

## Limits and errors

`Parse` and `Stringify` reject value trees deeper than **256** levels. Parse failures return `Result.Error(Message)`. Invalid `TomlValue` payloads passed to `Stringify` are programmer errors and raise a runtime diagnostic with a construction hint.

## Implementation (contributors)

| Concern | Location |
| --- | --- |
| Registration | [`loaded/toml.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/toml.rs) |
| Runtime | [`toml.rs`](../../../../crates/fpas-std/src/toml.rs) |
| Compiler intrinsic catalog | [`intrinsic_catalog.rs`](../../../../crates/fpas-compiler/src/intrinsic_catalog.rs) |
| Intrinsics | [`intrinsic/toml.rs`](../../../../crates/fpas-bytecode/src/intrinsic/toml.rs) |

## See also

- [`Std.Toml.Fields`](toml-fields.md) — typed key access for parsed tables
- [Text and parsing index](README.md)
- [Standard library index](../README.md)
