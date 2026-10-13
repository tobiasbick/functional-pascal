# `Std.Toml`

Parse and stringify TOML 1.0 documents with an explicit Functional Pascal value representation.

```pascal
program Example;

uses Std.Console, Std.Toml;

begin
  const Parsed: result of (TomlValue, string) := Parse('[project]' + string.Chr(10) + 'name = ''demo''');
  case Parsed of
    when Ok(const Value):
      WriteLn(Stringify(Value));
    when Error(const Message):
      WriteLn(Message);
  end case;
end.
```

## Importing and names

After `uses Std.Toml;` use short names (`TomlValue`, `Parse`, `Stringify`) or qualified names such as `Std.Toml.Parse`.

`TomlValue` is an enum. Tables use `dict of string to TomlValue`; arrays use `array of TomlValue`.

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
  ArrayValue(Items: array of TomlValue);
  Table(Fields: dict of string to TomlValue);
end enum;
```

`Datetime` preserves the TOML date, time, local date-time, or offset date-time spelling returned by the parser. Its `Value` must be a valid TOML date/time string when passed to `Stringify`.

## `Parse`

```pascal
function Parse(Text: string): Result of (TomlValue, string);
```

Parses a TOML document. Valid input returns `Ok(TomlValue)`; syntax errors return `Error(Message)` rather than aborting the program. TOML documents are tables at the root, so successful `Parse` results always have the `TomlValue.Table` variant.

Strings, signed 64-bit integers, finite floating-point values, booleans, date/time
values, arrays, tables, inline tables, and arrays of tables are represented.
TOML `inf` and `nan` values, including nested ones, return Error because FPAS
real values must be finite. Stringify also rejects nonfinite real payloads.

```pascal
const Parsed: result of (TomlValue, string) := Parse('title = ''example''' + string.Chr(10) + 'enabled = true' + string.Chr(10) + '[server]' + string.Chr(10) +
                                                 'port = 8080');
case Parsed of
  when Ok(const Value):
    if Value is TomlValue.Table(_) then
      WriteLn('parsed');
    end if;
  when Error(const Message):
    WriteLn('TOML error: ' + Message);
end case;
```

## `Stringify`

```pascal
function Stringify(Value: TomlValue): string;
```

Encodes a `TomlValue` tree as TOML. The supplied root must be a table because TOML documents have table roots. `Stringify` raises a runtime error for malformed manually constructed values, non-string table keys, invalid date/time text, or nesting deeper than 256 levels.

```pascal
const Value: TomlValue := TomlValue.Table([
  'project': TomlValue.Table([
    'name': TomlValue.String('demo'),
    'version': TomlValue.Integer(1)
  ])
]);
WriteLn(Stringify(Value));
```

## Limits and errors

FPAS value conversion and `Stringify` reject value trees deeper than **256**
levels. Before conversion, `Parse` uses the TOML dependency parser's separate,
shape-dependent recursion budget. Deeply nested arrays can exhaust that budget
well below 256 levels; the conversion limit is not a promise that every
shallower TOML input can be parsed. Parser and conversion failures both return
`Error(Message)`. Invalid `TomlValue` payloads passed to `Stringify` are
programmer errors and raise a runtime diagnostic with a construction hint.

## Implementation (contributors)

| Concern | Location |
| --- | --- |
| Registration | [`loaded/toml.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/toml.rs) |
| Runtime | [`toml/mod.rs`](../../../../crates/fpas-std/src/toml/mod.rs) |
| Compiler intrinsic catalog | [`intrinsic_catalog.rs`](../../../../crates/fpas-compiler/src/intrinsic_catalog.rs) |
| Intrinsics | [`intrinsic/toml.rs`](../../../../crates/fpas-bytecode/src/intrinsic/toml.rs) |

## See also

- [`Std.Toml.Fields`](toml-fields.md) — typed key access for parsed tables
- [Text and parsing index](README.md)
- [Standard library index](../README.md)
