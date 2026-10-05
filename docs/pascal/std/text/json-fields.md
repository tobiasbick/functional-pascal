# `Std.Json.Fields`

Strict typed access to the fields of parsed JSON objects. Every accessor returns
`Error(Message)` for a missing field or a value of the wrong JSON kind instead of
requiring a nested `case` per field.

```pascal
program Example;

uses Std.Console, Std.Json, Std.Json.Fields;

function ReadPort(Text: string): result of integer, string;
begin
  var Fields: dict of string to JsonValue := try ParseObject(Text);
  var Allowed: boolean := try RequireOnlyFields(Fields, ['host', 'port']);
  return IntegerField(Fields, 'port');
end;

begin
  case ReadPort('{"host":"localhost","port":8080}') of
    Ok(Port): WriteLn(Port);
    Error(Message): WriteLn('invalid configuration: ' + Message);
  end;
end.
```

## Importing and names

`uses Std.Json.Fields;` provides the accessors. Values and objects keep the
`Std.Json` types, so programs usually import both units.

## Quick reference

| Kind | Name | Notes |
|------|------|-------|
| function | `ParseObject(Text: string): Result of dict of string to JsonValue, string` | parses text whose root must be an object |
| function | `Field(Fields; Name: string): Result of JsonValue, string` | any present field |
| function | `StringField(Fields; Name: string): Result of string, string` | string field |
| function | `BooleanField(Fields; Name: string): Result of boolean, string` | boolean field |
| function | `NumberField(Fields; Name: string): Result of real, string` | number field |
| function | `IntegerField(Fields; Name: string): Result of integer, string` | exact integer in the 64-bit range |
| function | `ObjectField(Fields; Name: string): Result of dict of string to JsonValue, string` | nested object |
| function | `ArrayField(Fields; Name: string): Result of array of JsonValue, string` | array with any element kinds |
| function | `StringArrayField(Fields; Name: string): Result of array of string, string` | array whose elements are all strings |
| function | `RequireOnlyFields(Fields; Allowed: array of string): Result of boolean, string` | rejects unlisted field names |

`Fields` is always `dict of string to JsonValue`.

## Errors

Messages are neutral so callers can prefix their own context:

| Situation | Message |
|-----------|---------|
| missing field | `missing field <name>` |
| wrong kind | `field <name> must be a string` (or `a boolean`, `a number`, `an integer`, `an object`, `an array`, `an array of strings`) |
| integer outside `-9223372036854775808..9223372036854775807` | `field <name> is outside the integer range` |
| unlisted field | `unexpected field <name>` |
| root is not an object | `JSON root must be an object` |

`ParseObject` returns `Std.Json.Parse` errors unchanged.

## `IntegerField`

JSON numbers are `real` values. `IntegerField` accepts a number only when it has no
fractional part and lies in the signed 64-bit range. Unlike calling `Trunc` on an
untrusted value, it never raises a runtime error, so it is safe for decoding input
received from files or the network.

## `RequireOnlyFields`

Returns `Ok(true)` when every field name of `Fields` appears in `Allowed`, otherwise
`Error` for the first unlisted name in insertion order. It does not check that listed
fields are present; use the typed accessors for required fields.

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| FPAS implementation | [`Fields.fpas`](../../../../lib/Std/Json/Fields.fpas) |
| Regression tests | [`json_fields_typed_access_test.fpas`](../../../../tests/stdlib/json/json_fields_typed_access_test.fpas) |

## See also

- [`Std.Json`](json.md)
- [Text and parsing index](README.md)
- [Standard library index](../README.md)
