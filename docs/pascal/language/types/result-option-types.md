# Result and Option types

`Result of T, E` represents either a successful value of type `T` or an error value of type `E`.
`Option of T` represents either a present value of type `T` or the absence of a value.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`type_expr` — `result` / `option`).

```pascal
var Success: result of integer, string := Ok(42);
var Failure: result of integer, string := Error('not found');
var Present: option of integer := Some(7);
var Missing: option of integer := None;
```

Use `case` destructuring to handle both forms:

```pascal
uses Std.Console as Console;
uses Std.Conv as Conv;

case Success of
  when Ok(Value):
    Console.WriteLn(Conv.IntToStr(Value));
  when Error(Message):
    Console.WriteLn(Message);
end case;

case Console.Present of
  when Some(Value):
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
- [`Std.Results`](../../std/result/result.md), [`Std.Options`](../../std/result/option.md)
