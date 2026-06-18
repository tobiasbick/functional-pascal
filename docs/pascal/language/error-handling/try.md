# Try operator

`try` propagates errors automatically. If the expression is `Error` (for `Result`) or `None` (for `Option`), the enclosing function returns that value immediately. Otherwise, the inner value is unwrapped:

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`try` expression).

```pascal
function Process(A: integer; B: integer): Result of string, string;
begin
  var Quotient: integer := try Divide(A, B);
  return Ok(IntToStr(Quotient))
end;
```

`try` also works with `Option`:

```pascal
function FirstPositive(Items: array of integer): Option of integer;
begin
  var Idx: integer := try FindIndex(Items, 1);
  return Some(Items[Idx])
end;
```

## See also

- [Result](result.md)
- [Option](option.md)
- [Operators — `try` precedence](../basics/operators.md#operator-precedence)
