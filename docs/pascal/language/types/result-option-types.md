# Result and Option types

The code fragments use `uses Std.Text.Format as Conv, Std.Console as Console;`.

`Some`, `Ok` and `Error` accept one positional payload or the case-insensitive
label `Value`, for example `Some(Value := 3)` and `Error(Value := 'missing')`.
The error payload uses the declared error type. These labels also work in
patterns. `None` has no payload and uses no parentheses. Named payload labels
do not add field-access operations.

`Result of (T, E)` represents either a successful value of type `T` or an error value of type `E`.
`Option of T` represents either a present value of type `T` or the absence of a value.

`Result` always takes two parenthesized type arguments. Nested types use the
same form, for example `Result of (Option of integer, Result of (integer, string))`.
Single-argument `Option` types do not put parentheses around their type argument.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`type_expr` — `result` / `option`).

```pascal id=example-1 fragment context=body
const Success: result of (integer, string) := Ok(42);
const Failure: result of (integer, string) := Error('not found');
const Present: option of integer := Some(7);
const Missing: option of integer := None;
```

Use `case` destructuring to handle both forms:

```pascal id=example-2 fragment context=result-option
case Success of
  when Ok(const Value):
    Console.WriteLn(Conv.IntToStr(Value));
  when Error(const Message):
    Console.WriteLn(Message);
end case;

case Present of
  when Some(const Value):
    Console.WriteLn(Conv.IntToStr(Value));
  when None:
    Console.WriteLn('empty');
end case;
```

Use `try` to propagate `Error(...)` and `None` automatically from functions that return
`Result` or `Option`. For propagation rules, combinators, and standard-library helpers, see
[Error handling](../error-handling/README.md).

## See also

- [Error handling](../error-handling/README.md)
- [Pattern matching](../pattern-matching/README.md)
- [`Result operations`](result-operations.md), [`Option operations`](option-operations.md)
