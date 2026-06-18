# Result and Option types

`Result of T, E` represents either a successful value of type `T` or an error value of type `E`.
`Option of T` represents either a present value of type `T` or the absence of a value.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`type_expr` — `result` / `option`).

```pascal
var Success: Result of integer, string := Ok(42);
var Failure: Result of integer, string := Error('not found');

var Present: Option of integer := Some(7);
var Missing: Option of integer := None;
```

Use `case` destructuring to handle both forms:

```pascal
case Success of
  Ok(Value): WriteLn(IntToStr(Value));
  Error(Message): WriteLn(Message)
end;

case Present of
  Some(Value): WriteLn(IntToStr(Value));
  None: WriteLn('empty')
end;
```

Use `try` to propagate `Error(...)` and `None` automatically from functions that return
`Result` or `Option`. For propagation rules, combinators, and standard-library helpers, see
[Error handling](../../07-error-handling.md).

## See also

- [Error handling](../../07-error-handling.md)
- [Pattern matching](../pattern-matching/README.md)
- [`Std.Result`](../../std/result.md), [`Std.Option`](../../std/option.md)
