# `Std.Json.Fields`

Strict typed access to the fields of parsed JSON objects. Every accessor returns
`Error(Cause)` for a missing field or a value of the wrong JSON kind instead of
requiring a nested `case` per field.

```pascal id=example-1
program Example;

uses Std.Console as Console, Std.Json as Json, Std.Json.Fields as JsonFields;

function ReadPort(Text: string): result of (integer, JsonFields.FieldError);
begin
  const Fields: dict of string to Json.JsonValue := try JsonFields.ParseObject(Text);
  const Allowed: boolean := try JsonFields.RequireOnlyFields(Fields, ['host', 'port']);
  return JsonFields.IntegerField(Fields, 'port');
end function;

begin
  case ReadPort('{"host":"localhost","port":8080}') of
    when Ok(const Port):
      Console.WriteLn(Port);
    when Error(const Message):
      Console.WriteLn('invalid configuration: ' + Message.Message());
  end case;
end.
```

## Importing and names

`uses Std.Json.Fields as Fields;` provides the accessors through `Fields.Name`. Values and objects keep the
`Std.Json` types, so programs usually import both units.

## Quick reference

| Kind | Name | Notes |
|------|------|-------|
| function | `ParseObject(Text: string): Result of (dict of string to JsonValue, JsonFields.FieldError)` | parses text whose root must be an object |
| function | `Field(Fields; Name: string): Result of (JsonValue, JsonFields.FieldError)` | any present field |
| function | `StringField(Fields; Name: string): Result of (string, JsonFields.FieldError)` | string field |
| function | `BooleanField(Fields; Name: string): Result of (boolean, JsonFields.FieldError)` | boolean field |
| function | `NumberField(Fields; Name: string): Result of (real, JsonFields.FieldError)` | number field |
| function | `IntegerField(Fields; Name: string): Result of (integer, JsonFields.FieldError)` | exact integer in the 64-bit range |
| function | `ObjectField(Fields; Name: string): Result of (dict of string to JsonValue, JsonFields.FieldError)` | nested object |
| function | `ArrayField(Fields; Name: string): Result of (array of JsonValue, JsonFields.FieldError)` | array with any element kinds |
| function | `StringArrayField(Fields; Name: string): Result of (array of string, JsonFields.FieldError)` | array whose elements are all strings |
| function | `RequireOnlyFields(Fields; Allowed: array of string): Result of (boolean, JsonFields.FieldError)` | rejects unlisted field names |

`Fields` is always `dict of string to JsonValue`.

## Errors

All fallible helpers return `FieldError`. Match its variants to make decisions;
use `Message()` or direct printing at an output boundary.

| Situation | Variant |
| --- | --- |
| invalid JSON text | `ParseFailure(Cause)` with the original `Json.JsonError` |
| root is not an object | `ExpectedObject` |
| missing field | `MissingField(Name)` |
| wrong kind | `WrongType(Name, Expected)` |
| invalid numeric value | `InvalidValue(Name, Reason)` |
| unlisted field | `UnknownField(Name)` |

`ParseObject` wraps parser failures with `MapError(FieldError.ParseFailure)`,
preserving their typed cause. Local validation failures have no invented parser
cause. Unknown-field selection follows existing dictionary iteration order.

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
