# Result and Option types

`Result of T, E` represents either a successful value of type `T` or an error value of type `E`.
`Option of T` represents either a present value of type `T` or the absence of a value.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`type_expr` — `result` / `option`).

```pascal
const Success: Result of integer, string := Ok(42);
const Failure: Result of integer, string := Error('not found');

const Present: Option of integer := Some(7);
const Missing: Option of integer := None;
```

Use `case` destructuring to handle both forms:

```pascal
case Success of
  when Ok(const Value):
    WriteLn(IntToStr(Value));
  when Error(const Message):
    WriteLn(Message);
end case;

case Present of
  when Some(const Value):
    WriteLn(IntToStr(Value));
  when None:
    WriteLn('empty');
end case;
```

Use `try` to propagate `Error(...)` and `None` automatically from functions that return
`Result` or `Option`. For propagation rules, combinators, and standard-library helpers, see
[Error handling](../error-handling/README.md).

## See also

- [Error handling](../error-handling/README.md)
- [Pattern matching](../pattern-matching/README.md)
- [`Result operations`](result-operations.md), [`Option operations`](option-operations.md)
