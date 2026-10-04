# Result and Option types

`Result of (T, E)` represents either a successful value of type `T` or an error value of type `E`.
`Option of (T)` represents either a present value of type `T` or the absence of a value.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`type_expr` — `result` / `option`).

```pascal
const Success: result of (integer, string) := Result.Ok(42);
const Failure: result of (integer, string) := Result.Error('not found');
const Present: option of (integer) := Option.Some(7);
const Missing: option of (integer) := Option.None;
```

Use `case` destructuring to handle both forms:

```pascal
uses Std.Console as Console;
uses Std.Conv as Conv;

case Success of
  when Result.Ok(const Value):
    Console.WriteLn(Conv.IntToStr(Value));
  when Result.Error(const Message):
    Console.WriteLn(Message);
end case;

case Console.Present of
  when Option.Some(const Value):
    Console.WriteLn(Conv.IntToStr(Value));
  when Option.None:
    Console.WriteLn('empty');
end case;
```

Use `try` to propagate `Result.Error(...)` and `Option.None` automatically from functions that return
`Result` or `Option`. For propagation rules, combinators, and standard-library helpers, see
[Error handling](../error-handling/README.md).

## See also

- [Error handling](../error-handling/README.md)
- [Pattern matching](../pattern-matching/README.md)
- [`Std.Results`](../../std/result/result.md), [`Std.Options`](../../std/result/option.md)
