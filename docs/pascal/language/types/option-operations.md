# Option operations

The code fragments use `uses Std.Text.Format as Conv, Std.Console as Console;`.

Helper functions for `Option of T` values. See [Error handling](../error-handling/README.md) for the type itself, constructors (`Some`, `None`), the `try` operator, and `case` destructuring.

```pascal id=example-1
program Example;

uses Std.Console as Console;

begin
  const O: option of integer := Some(7);
  Console.WriteLn(O.Unwrap());
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
| `Value.Map(F: function(V: T): U): Option of U` | Transform the present value |
| `Value.AndThen(F: function(V: T): Option of U): Option of U` | Chain an optional operation |
| `Value.OrElse(F: function(): Option of T): Option of T` | Invoke a fallback only for `None` |
| `Value.Unwrap(): T` | Extract the value; panic for `None` |

## `Value.Unwrap(): T`

Extracts the value from `Some(value)`. **Runtime error** if `O` is `None`.
Use Unwrap when abort is intended; handle expected absence with case.

```pascal id=example-2 fragment context=body-console
const O: option of integer := Some(7);
Console.WriteLn(O.Unwrap()); // 7
```

---

## Variant tests and plain defaults

`Value is Some(_)` and `Value is None` test the variant without binding data.
Case binds the payload and handles absence locally. A plain default is evaluated
only in its selected arm:

```pascal id=example-3 fragment context=body
const O: option of integer := None;
const Value: integer := case O of
  when Some(const Present): Present;
  when None: -1;
end case;
```

The selector evaluates once. Ordinary function arguments evaluate eagerly;
work in a case arm runs only when that arm is selected. To run default work
unconditionally, compute it before case and refer to the saved value in the
None arm. Map, AndThen, and OrElse keep Option wrapping.

## `Value.Map(F: function(V: T): U): Option of U`

Transforms the `Some` value with `F`. If `O` is `None`, returns `None`.

```pascal id=example-4 fragment context=declarations-conv
function TripleToString(V: integer): string;
begin
  return Conv.IntToStr(V * 3);
end function;

const O: option of integer := Some(7);
const M: option of string := O.Map(TripleToString);
// M = Some('21')
```

---

## `Value.AndThen(F: function(V: T): Option of U): Option of U`

Calls `F` with the `Some` value. `F` returns a new `Option`, enabling chained lookups. If `O` is `None`, returns `None`.

```pascal id=example-5 fragment context=declarations-conv
function PositiveToOptionString(V: integer): option of string;
begin
  if V > 0 then
    return Some(Conv.IntToStr(V));
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
The callback body runs only for None. The expression supplying F is an ordinary
eager argument and is evaluated once even for Some. OrElse returns Option of T;
use case for a plain replacement value. Callback runtime failures propagate.

```pascal id=example-6 fragment context=declarations
function Fallback99(): option of integer;
begin
  return Some(99);
end function;

const O: option of integer := None;
const M: option of integer := O.OrElse(Fallback99);
// M = Some(99)
```

---

## Implementation (contributors)

| Concern | Location |
|---------|-----------|
| Runtime logic | [`result_option.rs`](../../../../crates/fpas-runtime/src/stdlib/result_option.rs) |
| Direct VM success path | [`intrinsics/unary.rs`](../../../../crates/fpas-runtime/src/execution/vm/execute/intrinsics/unary.rs) |
| Type checking | [`std_registry/builtins/result_option.rs`](../../../../crates/fpas-sema/src/std_registry/builtins/result_option.rs) |
| Registration | [`std_registry/native/option.rs`](../../../../crates/fpas-sema/src/std_registry/native/option.rs) |

## See also

- [Result and Option index](../../std/result/README.md)
- [Standard library index](../../std/README.md)
