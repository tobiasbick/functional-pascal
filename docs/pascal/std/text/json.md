# `Std.Encoding.Json`

JSON parsing and stringification with an explicit Functional Pascal value representation.

```pascal id=example-1
program Example;

uses Std.Console as Console, Std.Encoding.Json as Json;

begin
  const R: result of (Json.JsonValue, Json.JsonError) := Json.Parse('{"ok":true}');
  case R of
    when Ok(const Value):
      Console.WriteLn(Json.Stringify(Value));
    when Error(const Message):
      Console.WriteLn(Message);
  end case;
end.
```


## Importing and names

Import with `uses Std.Encoding.Json as Json;` and access public symbols as `Json.Name`. User enum variants require `Json.Type.Variant`.

`JsonValue` is an enum. Use `JsonValue.String('text')`, `JsonValue.ArrayValue([...])`, and similar constructors for new JSON values.

---

## Quick reference

| Kind | Name | Notes |
|------|------|-------|
| type | `JsonValue` | JSON tree representation |
| function | `Parse(Text: string): Result of (JsonValue, Json.JsonError)` | parses JSON text; parse failures are `Error(Cause)` |
| function | `Stringify(Value: JsonValue): string` | serializes a JSON value to compact JSON text |

### `JsonValue`

```pascal id=example-2 fragment context=declarations
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

```text
function Parse(Text: string): Result of (Json.JsonValue, Json.JsonError);
```

Object members become dictionary entries in document order. Rejects duplicate object member names with `Error(Cause)` identifying the name and its location. Names are compared after decoding escapes and are case-sensitive. Each object has its own name set, including objects nested in arrays.

Parses JSON text. Accepted JSON returns `Ok(JsonValue)`. Invalid JSON returns `Error(Cause)` instead of aborting the program.

```pascal id=example-4 fragment context=body-console-json
const R: result of (Json.JsonValue, Json.JsonError) := Json.Parse('[1, true, null]');
case R of
  when Ok(const Value):
    Console.WriteLn(Json.Stringify(Value));
  when Error(const Message):
    Console.WriteLn('JSON error: ' + Message.Message());
end case;
```

### `Stringify`

```text
function Stringify(Value: Json.JsonValue): string;
```

Serializes a `JsonValue` to compact JSON text. Object members are written in the insertion order of their dictionary.

A number with an integral value up to 2^53 in magnitude is written without a fraction, so
`JsonValue.Number(2.0)` becomes `2`. Other numbers keep their fraction or exponent, and negative
zero stays `-0.0`. JSON does not distinguish these forms, so `Parse` reads either back as the same
real value.

```pascal id=example-6 fragment context=body-console-json
const Value: Json.JsonValue := Json.JsonValue.ArrayValue([
  Json.JsonValue.Bool(true),
  Json.JsonValue.NullValue,
  Json.JsonValue.String('hi'),
  Json.JsonValue.Number(1.5)
]);
Console.WriteLn(Json.Stringify(Value)); // [true,null,"hi",1.5]
```

Malformed runtime payloads, such as an enum value pretending to be `JsonValue`, raise a runtime error. Normal parse failures should be handled through the `Result` returned by `Parse`.

FPAS value conversion and `Stringify` have a **256-level** nesting limit;
conversion failures make `Parse` return `Error(Cause)`, and stringification
failures raise a runtime error. This is not the effective JSON parser limit:
`Parse` also uses serde_json's default **128-step recursion budget** and may
reject nested arrays or objects before reaching the FPAS conversion limit.
Those parser failures also return `Error(Cause)`.

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Registration | [`loaded/json.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/json.rs) |
| Runtime | [`json.rs`](../../../../crates/fpas-runtime/src/stdlib/json.rs) |
| Compiler intrinsic catalog | [`intrinsic_catalog.rs`](../../../../crates/fpas-compiler/src/intrinsic_catalog.rs) |
| Intrinsics | [`intrinsic/json.rs`](../../../../crates/fpas-code/src/intrinsic/json.rs) |

## See also

- [`Std.Encoding.Json.Fields`](json-fields.md) — typed field access for parsed objects
- [Text and parsing index](README.md)
- [Standard library index](../README.md)

## Typed failures

`Json.JsonError.InvalidSyntax(Line, Column, Detail)` retains parser diagnostics. Line and Column are `Option of integer`, present only when the parser supplies a position (one-based); unsupported/nonfinite document values have no invented position. Stringify keeps its existing non-Result contract and runtime validation.
