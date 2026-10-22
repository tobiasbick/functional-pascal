# Result operations

The code fragments use `uses Std.Conv as Conv, Std.Console as Console;`.

Helper functions for `Result of (T, E)` values. See [Error handling](../error-handling/README.md) for the type itself, constructors (`Ok`, `Error`), the `try` operator, and `case` destructuring.

```pascal id=example-1
program Example;

uses Std.Console as Console;

begin
  const R: result of (integer, string) := Ok(42);
  Console.WriteLn(R.Unwrap());
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
| `Value.Map(F: function(V: T): U): Result of (U, E)` | Transform the successful value |
| `Value.MapError(F: function(Cause: E): E2): Result of (T, E2)` | Transform the error payload without recovering or flattening |
| `Value.AndThen(F: function(V: T): Result of (U, E)): Result of (U, E)` | Chain an operation with the same error type |
| `Value.OrElse(F: function(Err: E): Result of (T, E2)): Result of (T, E2)` | Invoke error recovery; may change the error type |
| `Value.Unwrap(): T` | Extract the value; panic for `Error` |

## `Value.Unwrap(): T`

Extracts the value from `Ok(value)`. **Runtime error** if `R` is `Error`.
Use Unwrap when abort is intended; handle expected errors with case or propagate
them with try inside a compatible Result-returning function.

```pascal id=example-2 fragment context=body-console
const R: result of (integer, string) := Ok(42);
Console.WriteLn(R.Unwrap()); // 42
```

---

## `Value.Map(F: function(V: T): U): Result of (U, E)`

Transforms the `Ok` value with `F`. If `R` is `Error`, returns it unchanged.

```pascal id=example-3 fragment context=declarations-conv
function DoubleToString(V: integer): string;
begin
  return Conv.IntToStr(V * 2);
end function;

const R: result of (integer, string) := Ok(21);
const M: result of (string, string) := R.Map(DoubleToString);
// M = Ok('42')
```

---

## `Value.MapError(F: function(Cause: E): E2): Result of (T, E2)`

Preserves an `Ok` value without invoking F. For `Error`, calls F exactly once
and wraps its result in `Error`. The receiver and callable expression are
evaluated once, in that order, even for `Ok`; creating captures may have effects.
Both error types are unrestricted. If F returns a Result, that Result stays the
nested error payload. Panics and contract failures remain runtime failures.

Payload enum and native-error constructors can be passed directly. Their field
names remain the labels of direct constructor calls; a callable binding uses its
own declared labels. Use an ordinary captured function when extra context is needed.

```pascal id=example-6 run
program MapFailure;
uses Std.Test as Test;
type ConfigError = error
  Invalid(Text: string);
end error;
begin
  const Input: result of (integer, string) := Error('setting');
  const Mapped := Input.MapError(ConfigError.Invalid);
  Test.AssertTrue(Mapped is Error(ConfigError.Invalid('setting')));
end.
```

MapError does not recover to Ok, flatten nested Results or implicitly convert
error types. Apply it before `try` when the caller returns a different error type;
`try` itself still requires compatible error types.

---

## `Value.AndThen(F: function(V: T): Result of (U, E)): Result of (U, E)`

Calls `F` with the `Ok` value. `F` returns a new `Result`, enabling chained fallible operations. If `R` is `Error`, returns it unchanged.

```pascal id=example-4 fragment context=declarations-conv
function PositiveToResult(V: integer): result of (string, string);
begin
  if V > 0 then
    return Ok(Conv.IntToStr(V));
  else
    return Error('non-positive');
  end if;
end function;

const R: result of (integer, string) := Ok(10);
const M: result of (string, string) := R.AndThen(PositiveToResult);
// M = Ok('10')
```

---

## `Value.OrElse(F: function(Err: E): Result of (T, E2)): Result of (T, E2)`

Calls `F` with the `Error` value to attempt recovery. If `R` is `Ok`, returns it unchanged.

```pascal id=example-5 fragment context=declarations
function RecoverToZero(E: string): result of (integer, string);
begin
  return Ok(0);
end function;

const R: result of (integer, string) := Error('oops');
const M: result of (integer, string) := R.OrElse(RecoverToZero);
// M = Ok(0)
```

---

## Implementation (contributors)

| Concern | Location |
|---------|-----------|
| Runtime logic | [`result_option.rs`](../../../../crates/fpas-std/src/result_option.rs) |
| Direct VM success path | [`intrinsics/unary.rs`](../../../../crates/fpas-vm/src/vm/execute/intrinsics/unary.rs) |
| Type checking | [`std_registry/builtins/result_option.rs`](../../../../crates/fpas-sema/src/std_registry/builtins/result_option.rs) |
| Registration | [`std_registry/native/result.rs`](../../../../crates/fpas-sema/src/std_registry/native/result.rs) |

## See also

- [Result and Option index](../../std/result/README.md)
- [Standard library index](../../std/README.md)
