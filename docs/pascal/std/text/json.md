# `Std.Json`

JSON parsing and stringification with an explicit Functional Pascal value representation.

```pascal
program Example;

uses Std.Console, Std.Json;

begin
  const R: result of (JsonValue, string) := Parse('{"ok":true}');
  case R of
    when Ok(const Value):
      WriteLn(Stringify(Value));
    when Error(const Message):
      WriteLn(Message);
  end case;
end.
```


## Importing and names

After `uses Std.Json;` use short names (`JsonValue`, `Parse`, `Stringify`) or qualified names (`Std.Json.JsonValue`, `Std.Json.Parse`, `Std.Json.Stringify`).

`JsonValue` is an enum. Use `JsonValue.String('text')`, `JsonValue.ArrayValue([...])`, and similar constructors for new JSON values.

---

## Quick reference

| Kind | Name | Notes |
|------|------|-------|
| type | `JsonValue` | JSON tree representation |
| function | `Parse(Text: string): Result of (JsonValue, string)` | parses JSON text; parse failures are `Error(Message)` |
| function | `Stringify(Value: JsonValue): string` | serializes a JSON value to compact JSON text |

### `JsonValue`

```pascal
type JsonValue = enum
  NullValue;
  Bool(Value: boolean);
  Number(Value: real);
  String(Value: string);
  ArrayValue(Items: array of JsonValue);
  Object(Fields: dict of string to JsonValue);
end enum;
```

JSON `null` maps to `JsonValue.NullValue`. The constructor has no fields and
serializes back to the JSON text `null`. Objects use `dict of string to JsonValue`.
Arrays use `array of JsonValue`.

---

## Detailed reference

### `Parse`

```pascal
function Parse(Text: string): Result of (JsonValue, string);
```

Object members become dictionary entries in document order. Rejects duplicate object member names with `Error(Message)` identifying the name and its location. Names are compared after decoding escapes and are case-sensitive. Each object has its own name set, including objects nested in arrays.

Parses JSON text. Accepted JSON returns `Ok(JsonValue)`. Invalid JSON returns `Error(Message)` instead of aborting the program.

```pascal
const R: result of (JsonValue, string) := Std.Json.Parse('[1, true, null]');
case R of
  when Ok(const Value):
    WriteLn(Std.Json.Stringify(Value));
  when Error(const Message):
    WriteLn('JSON error: ' + Message);
end case;
```

### `Stringify`

```pascal
function Stringify(Value: JsonValue): string;
```

Serializes a `JsonValue` to compact JSON text. Object members are written in the insertion order of their dictionary.

A number with an integral value up to 2^53 in magnitude is written without a fraction, so
`JsonValue.Number(2.0)` becomes `2`. Other numbers keep their fraction or exponent, and negative
zero stays `-0.0`. JSON does not distinguish these forms, so `Parse` reads either back as the same
real value.

```pascal
const Value: JsonValue := JsonValue.ArrayValue([
  JsonValue.Bool(true),
  JsonValue.NullValue,
  JsonValue.String('hi'),
  JsonValue.Number(1.5)
]);
WriteLn(Std.Json.Stringify(Value));  // [true,null,"hi",1.5]
```

Malformed runtime payloads, such as an enum value pretending to be `JsonValue`, raise a runtime error. Normal parse failures should be handled through the `Result` returned by `Parse`.

Nesting deeper than **256** levels is rejected: `Parse` returns `Error(Message)` and `Stringify` raises a runtime error.

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Registration | [`loaded/json.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/json.rs) |
| Runtime | [`json.rs`](../../../../crates/fpas-std/src/json.rs) |
| Compiler intrinsic catalog | [`intrinsic_catalog.rs`](../../../../crates/fpas-compiler/src/intrinsic_catalog.rs) |
| Intrinsics | [`intrinsic/json.rs`](../../../../crates/fpas-bytecode/src/intrinsic/json.rs) |

## See also

- [`Std.Json.Fields`](json-fields.md) — typed field access for parsed objects
- [Text and parsing index](README.md)
- [Standard library index](../README.md)
