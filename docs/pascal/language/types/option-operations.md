# Option operations

Helper functions for `Option of T` values. See [Error handling](../error-handling/README.md) for the type itself, constructors (`Some`, `None`), the `try` operator, and `case` destructuring.

```pascal
program Example;
uses Std.Console;
begin
  const O: Option of integer := Some(7);
  WriteLn(O.Unwrap());
end.
```


## Availability and call forms

These built-in type operations are always available without imports. Use
dot calls with positional or fully named explicit arguments. The static
receiver type and operation name select one catalog entry; ordinary free
functions with the same name do not affect this selection.

---

## Quick reference

| Operation | Result / behavior |
| --- | --- |
| `Value.IsSome(): boolean` | Whether a value is present |
| `Value.IsNone(): boolean` | Whether the option is absent |
| `Value.Map(F: function(V: T): U): Option of U` | Transform the present value |
| `Value.AndThen(F: function(V: T): Option of U): Option of U` | Chain an optional operation |
| `Value.OrElse(F: function(): Option of T): Option of T` | Invoke a fallback only for `None` |
| `Value.Unwrap(): T` | Extract the value; panic for `None` |
| `Value.UnwrapOr(Default: T): T` | Extract the value or use the default |


## `Value.Unwrap(): T`

Extracts the value from `Some(value)`. **Runtime error** if `O` is `None`.

```pascal
const O: Option of integer := Some(7);
WriteLn(O.Unwrap());                             // 7
```

---

## `Value.UnwrapOr(Default: T): T`

Extracts the value from `Some(value)`, or returns `Default` if `O` is `None`.

```pascal
const O: Option of integer := None;
WriteLn(O.UnwrapOr(-1));                      // -1
```

---

## `Value.IsSome(): boolean`

Returns `true` if `O` is a `Some` variant.

```pascal
const O: Option of integer := Some(7);
WriteLn(O.IsSome());                             // true
```

---

## `Value.IsNone(): boolean`

Returns `true` if `O` is `None`.

```pascal
const O: Option of integer := None;
WriteLn(O.IsNone());                             // true
```

---

## `Value.Map(F: function(V: T): U): Option of U`

Transforms the `Some` value with `F`. If `O` is `None`, returns `None`.

```pascal
function TripleToString(V: integer): string;
begin
  return IntToStr(V * 3);
end function;

const O: Option of integer := Some(7);
const M: Option of string := O.Map(TripleToString);
// M = Some('21')
```

---

## `Value.AndThen(F: function(V: T): Option of U): Option of U`

Calls `F` with the `Some` value. `F` returns a new `Option`, enabling chained lookups. If `O` is `None`, returns `None`.

```pascal
function PositiveToOptionString(V: integer): option of string;
begin
  if V > 0 then
    return Some(IntToStr(V));
  else
    return None;
  end if;
end function;

const O: option of integer := Some(5);
const M: option of string := O.AndThen(PositiveToOptionString);
// M = Some('5')
```

---

## `Value.OrElse(F: function(): Option of T): Option of T`

Calls `F` to provide a fallback when `O` is `None`. If `O` is `Some`, returns it unchanged.

```pascal
function Fallback99(): Option of integer;
begin
  return Some(99);
end function;

const O: Option of integer := None;
const M: Option of integer := O.OrElse(Fallback99);
// M = Some(99)
```

---

## Implementation (contributors)

| Concern | Location |
|---------|-----------|
| Runtime logic | [`result_option.rs`](../../../../crates/fpas-std/src/result_option.rs) |
| Type checking | [`std_registry/builtins/result_option.rs`](../../../../crates/fpas-sema/src/std_registry/builtins/result_option.rs) |
| Registration | [`std_registry/native/option.rs`](../../../../crates/fpas-sema/src/std_registry/native/option.rs) |

## See also

- [Result and Option index](../../std/result/README.md)
- [Standard library index](../../std/README.md)
