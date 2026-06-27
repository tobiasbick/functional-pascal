# `Std.Json`

JSON parsing and stringification with an explicit Functional Pascal value representation.

```pascal
program Example;
uses Std.Console, Std.Json;
begin
  var R: Result of JsonValue, string := Parse('{"ok":true}');
  case R of
    Ok(Value): WriteLn(Stringify(Value));
    Error(Message): WriteLn(Message)
  end
end.
```


## Importing and names

After `uses Std.Json;` use short names (`JsonValue`, `Parse`, `Stringify`) or qualified names (`Std.Json.JsonValue`, `Std.Json.Parse`, `Std.Json.Stringify`).

`JsonValue` is an enum. Use `JsonValue.String('text')`, `JsonValue.Array([...])`, and similar constructors for new JSON values.

---

## Quick reference

| Kind | Name | Notes |
|------|------|-------|
| type | `JsonValue` | JSON tree representation |
| function | `Parse(Text: string): Result of JsonValue, string` | parses JSON text; parse failures are `Error(Message)` |
| function | `Stringify(Value: JsonValue): string` | serializes a JSON value to compact JSON text |

### `JsonValue`

```pascal
type JsonValue = enum
  Null;
  Bool(Value: boolean);
  Number(Value: real);
  String(Value: string);
  Array(Items: array of JsonValue);
  Object(Fields: dict of string to JsonValue);
end;
```

JSON `null` maps to `JsonValue.Null`. Objects use `dict of string to JsonValue`. Arrays use `array of JsonValue`.

---

## Detailed reference

### `Parse`

```pascal
function Parse(Text: string): Result of JsonValue, string;
```

Parses JSON text. Valid JSON returns `Ok(JsonValue)`. Invalid JSON returns `Error(Message)` instead of aborting the program.

```pascal
var R: Result of JsonValue, string := Std.Json.Parse('[1, true, null]');
case R of
  Ok(Value): WriteLn(Std.Json.Stringify(Value));
  Error(Message): WriteLn('JSON error: ' + Message)
end
```

### `Stringify`

```pascal
function Stringify(Value: JsonValue): string;
```

Serializes a `JsonValue` to compact JSON text.

```pascal
var Value: JsonValue := JsonValue.Array([
  JsonValue.Bool(true),
  JsonValue.Null,
  JsonValue.String('hi'),
  JsonValue.Number(1.5)
]);
WriteLn(Std.Json.Stringify(Value))  { [true,null,"hi",1.5] }
```

Malformed runtime payloads, such as an enum value pretending to be `JsonValue`, raise a runtime error. Normal parse failures should be handled through the `Result` returned by `Parse`.

Nesting deeper than **256** levels is rejected: `Parse` returns `Error(Message)` and `Stringify` raises a runtime error.

## Implementation (contributors)

| Concern | Location |
|---------|----------|
| Registration | [`loaded/json.rs`](../../../../crates/fpas-sema/src/std_registry/loaded/json.rs) |
| Runtime | [`json.rs`](../../../../crates/fpas-std/src/json.rs) |
| Compiler | [`std_calls/json.rs`](../../../../crates/fpas-compiler/src/compiler/std_calls/json.rs) |
| Intrinsics | [`intrinsic/json.rs`](../../../../crates/fpas-bytecode/src/intrinsic/json.rs) |

## See also

- [Text and parsing index](README.md)
- [Standard library index](../README.md)
