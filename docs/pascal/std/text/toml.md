# `Std.Toml`

Parse and stringify TOML 1.0 documents with an explicit Functional Pascal value representation.

```pascal
program Example;

uses Std.Console, Std.Str, Std.Toml;

begin
  var Parsed: result of TomlValue, string := Parse('[project]' + Chr(10) + 'name = ''demo''');
  case Parsed of
    Ok(Value): WriteLn(Stringify(Value));
    Error(Message): WriteLn(Message)
  end
end.
```

## Importing and names

After `uses Std.Toml;` use short names (`TomlValue`, `Parse`, `Stringify`) or qualified names such as `Std.Toml.Parse`.

`TomlValue` is an enum. Tables use `dict of string to TomlValue`; arrays use `array of TomlValue`.

## Quick reference

| Kind | Name | Notes |
| --- | --- | --- |
| type | `TomlValue` | TOML value tree |
| function | `Parse(Text: string): Result of TomlValue, string` | Parses one TOML document |
| function | `Stringify(Value: TomlValue): string` | Encodes a TOML value tree |

### `TomlValue`

```pascal
type TomlValue = enum
  String(Value: string);
  Integer(Value: integer);
  Float(Value: real);
  Boolean(Value: boolean);
  Datetime(Value: string);
  Array(Items: array of TomlValue);
  Table(Fields: dict of string to TomlValue);
end;
```

`Datetime` preserves the TOML date, time, local date-time, or offset date-time spelling returned by the parser. Its `Value` must be a valid TOML date/time string when passed to `Stringify`.

## `Parse`

```pascal
function Parse(Text: string): Result of TomlValue, string;
```

Parses a TOML document. Valid input returns `Ok(TomlValue)`; syntax errors return `Error(Message)` rather than aborting the program. TOML documents are tables at the root, so successful `Parse` results always have the `TomlValue.Table` variant.

All TOML 1.0 value kinds are represented: strings, signed 64-bit integers, floating-point values (including `inf` and `nan`), booleans, date/time values, arrays, tables, inline tables, and arrays of tables.

```pascal
var Parsed: result of TomlValue, string := Parse(
  'title = ''example''' + Chr(10) +
  'enabled = true' + Chr(10) +
  '[server]' + Chr(10) +
  'port = 8080'
);
case Parsed of
  Ok(TomlValue.Table(Fields)): WriteLn('parsed');
  Error(Message): WriteLn('TOML error: ' + Message)
end
```

## `Stringify`

```pascal
function Stringify(Value: TomlValue): string;
```

Encodes a `TomlValue` tree as TOML. The supplied root must be a table because TOML documents have table roots. `Stringify` raises a runtime error for malformed manually constructed values, non-string table keys, invalid date/time text, or nesting deeper than 256 levels.

```pascal
var Value: TomlValue := TomlValue.Table([
  'project': TomlValue.Table([
    'name': TomlValue.String('demo'),
    'version': TomlValue.Integer(1)
  ])
]);
WriteLn(Stringify(Value));
```

## Limits and errors

`Parse` and `Stringify` reject value trees deeper than **256** levels. Parse failures return `Error(Message)`. Invalid `TomlValue` payloads passed to `Stringify` are programmer errors and raise a runtime diagnostic with a construction hint.

## Implementation (contributors)

| Concern | Location |
| --- | --- |
| Registration | [`loaded/toml.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/toml.rs) |
| Runtime | [`toml.rs`](../../../../crates/fpas-std/src/toml.rs) |
| Compiler intrinsic catalog | [`intrinsic_catalog.rs`](../../../../crates/fpas-compiler/src/intrinsic_catalog.rs) |
| Intrinsics | [`intrinsic/toml.rs`](../../../../crates/fpas-bytecode/src/intrinsic/toml.rs) |

## See also

- [Text and parsing index](README.md)
- [Standard library index](../README.md)
