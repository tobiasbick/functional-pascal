# `Std.Toml.Fields`

Strict typed access to the keys of parsed TOML tables. Every accessor returns
`Error(Message)` for a missing key or a value of the wrong TOML kind instead of
requiring a nested `case` per key.

```pascal
program Example;

uses Std.Console as Console;
uses Std.Str as Str;
uses Std.Toml as Toml;
uses Std.Toml.Fields as TomlFields;

function ReadPort(Text: string): result of integer, string;
begin
  var Fields: dict of string to Toml.TomlValue := try TomlFields.ParseTable(Text);
  var Allowed: boolean := try TomlFields.RequireOnlyFields(Fields, ['host', 'port']);
  return TomlFields.IntegerField(Fields, 'port');
end function;

begin
  case ReadPort(('host = ''localhost''' + Str.Chr(10)) + 'port = 8080') of
    when Ok(Port):
      Console.WriteLn(Port);
    when Error(Message):
      Console.WriteLn('invalid configuration: ' + Message);
  end case;
end program;
```

## Importing and names

`uses Std.Toml.Fields as TomlFields;` provides the accessors. Values and tables keep the
`Std.Toml` types, so programs usually import both units.

## Quick reference

| Kind | Name | Notes |
|------|------|-------|
| function | `ParseTable(Text: string): Result of dict of string to TomlValue, string` | parses a document and returns its root table |
| function | `Field(Fields; Name: string): Result of TomlValue, string` | any present key |
| function | `StringField(Fields; Name: string): Result of string, string` | string key |
| function | `IntegerField(Fields; Name: string): Result of integer, string` | integer key; floats are rejected |
| function | `FloatField(Fields; Name: string): Result of real, string` | float key; integers are rejected |
| function | `BooleanField(Fields; Name: string): Result of boolean, string` | boolean key |
| function | `TableField(Fields; Name: string): Result of dict of string to TomlValue, string` | nested or inline table |
| function | `ArrayField(Fields; Name: string): Result of array of TomlValue, string` | array with any element kinds |
| function | `StringArrayField(Fields; Name: string): Result of array of string, string` | array whose elements are all strings |
| function | `RequireOnlyFields(Fields; Allowed: array of string): Result of boolean, string` | rejects unlisted key names |

`Fields` is always `dict of string to TomlValue`. Date/time values are read with `Field`
and matched as `TomlValue.Datetime`.

## Errors

Messages are neutral so callers can prefix their own context, such as the configuration
file path:

| Situation | Message |
|-----------|---------|
| missing key | `missing field <name>` |
| wrong kind | `field <name> must be a string` (or `an integer`, `a float`, `a boolean`, `a table`, `an array`, `an array of strings`) |
| unlisted key | `unexpected field <name>` |

`ParseTable` returns `Std.Toml.Parse` errors unchanged.

## `RequireOnlyFields`

Returns `Ok(true)` when every key of `Fields` appears in `Allowed`, otherwise `Error` for
the first unlisted key. It checks one table level; nested tables are checked separately.
It does not check that listed keys are present. This supports rejecting configuration
files written for an older schema.

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| FPAS implementation | [`Fields.fpas`](../../../../lib/Std/Toml/Fields.fpas) |
| Regression tests | [`toml_fields_typed_access_test.fpas`](../../../../tests/stdlib/toml/toml_fields_typed_access_test.fpas) |

## See also

- [`Std.Toml`](toml.md)
- [Text and parsing index](README.md)
- [Standard library index](../README.md)
