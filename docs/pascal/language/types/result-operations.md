# Result operations

Helper functions for `Result of T, E` values. See [Error handling](../error-handling/README.md) for the type itself, constructors (`Ok`, `Error`), the `try` operator, and `case` destructuring.

```pascal
program Example;
uses Std.Console;
begin
  const R: Result of integer, string := Ok(42);
  WriteLn(R.Unwrap());
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
| `Value.IsOk(): boolean` | Whether the result is successful |
| `Value.IsError(): boolean` | Whether the result is an error |
| `Value.Map(F: function(V: T): U): Result of U, E` | Transform the successful value |
| `Value.AndThen(F: function(V: T): Result of (U, E)): Result of U, E` | Chain an operation with the same error type |
| `Value.OrElse(F: function(Err: E): Result of (T, E2)): Result of T, E2` | Invoke error recovery; may change the error type |
| `Value.Unwrap(): T` | Extract the value; panic for `Error` |
| `Value.UnwrapOr(Default: T): T` | Extract the value or use the default |


## `Value.Unwrap(): T`

Extracts the value from `Ok(value)`. **Runtime error** if `R` is `Error`.

```pascal
const R: Result of integer, string := Ok(42);
WriteLn(R.Unwrap());                             // 42
```

---

## `Value.UnwrapOr(Default: T): T`

Extracts the value from `Ok(value)`, or returns `Default` if `R` is `Error`.

```pascal
const R: Result of integer, string := Error('oops');
WriteLn(R.UnwrapOr(0));                       // 0
```

---

## `Value.IsOk(): boolean`

Returns `true` if `R` is an `Ok` variant.

```pascal
const R: Result of integer, string := Ok(42);
WriteLn(R.IsOk());                               // true
```

---

## `Value.IsError(): boolean`

Returns `true` if `R` is an `Error` variant.

```pascal
const R: Result of integer, string := Error('fail');
WriteLn(R.IsError());                              // true
```

---

## `Value.Map(F: function(V: T): U): Result of U, E`

Transforms the `Ok` value with `F`. If `R` is `Error`, returns it unchanged.

```pascal
function DoubleToString(V: integer): string;
begin
  return IntToStr(V * 2);
end function;

const R: Result of integer, string := Ok(21);
const M: Result of string, string := R.Map(DoubleToString);
// M = Ok('42')
```

---

## `Value.AndThen(F: function(V: T): Result of (U, E)): Result of U, E`

Calls `F` with the `Ok` value. `F` returns a new `Result`, enabling chained fallible operations. If `R` is `Error`, returns it unchanged.

```pascal
function PositiveToResult(V: integer): result of string, string;
begin
  if V > 0 then
    return Ok(IntToStr(V));
  else
    return Error('non-positive');
  end if;
end function;

const R: result of integer, string := Ok(10);
const M: result of string, string := R.AndThen(PositiveToResult);
// M = Ok('10')
```

---

## `Value.OrElse(F: function(Err: E): Result of (T, E2)): Result of T, E2`

Calls `F` with the `Error` value to attempt recovery. If `R` is `Ok`, returns it unchanged.

```pascal
function RecoverToZero(E: string): Result of integer, string;
begin
  return Ok(0);
end function;

const R: Result of integer, string := Error('oops');
const M: Result of integer, string := R.OrElse(RecoverToZero);
// M = Ok(0)
```

---

## Implementation (contributors)

| Concern | Location |
|---------|-----------|
| Runtime logic | [`result_option.rs`](../../../../crates/fpas-std/src/result_option.rs) |
| Type checking | [`std_registry/builtins/result_option.rs`](../../../../crates/fpas-sema/src/std_registry/builtins/result_option.rs) |
| Registration | [`std_registry/native/result.rs`](../../../../crates/fpas-sema/src/std_registry/native/result.rs) |

## See also

- [Result and Option index](../../std/result/README.md)
- [Standard library index](../../std/README.md)
