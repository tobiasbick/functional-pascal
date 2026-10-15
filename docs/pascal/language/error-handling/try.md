# Try operator

The code fragments use `uses Std.Conv as Conv;`.

`try` propagates errors automatically. If the expression is `Error` (for `Result`) or `None` (for `Option`), the enclosing function returns that value immediately. Otherwise, the inner value is unwrapped:

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`try` expression).

```pascal id=example-1 fragment context=try-divide
function Process(A: integer; B: integer): result of (string, string);
begin
  const Quotient: integer := try Divide(A, B);
  return Ok(Conv.IntToStr(Quotient));
end function;
```

`try` also works with `Option`. This function reads the first occurrence of one,
or propagates `None` when one is absent:

```pascal id=example-2 fragment context=declarations
function FindOne(Items: array of integer): option of integer;
begin
  const Idx: integer := try Items.IndexOf(1);
  return Some(Items[Idx]);
end function;
```

## Compatibility and evaluation

| Operand | Enclosing function return type | Behavior |
|---------|--------------------------------|----------|
| `Result of (T, E)` | `Result of (U, F)` with compatible error types | Extract Ok(T), or return Error(E) immediately. T and U may differ. |
| `Option of T` | `Option of U` | Extract Some(T), or return None immediately. T and U may differ. |
| Result | Result with an incompatible error type | Rejected statically. |
| Result | Option | Rejected statically. |
| Option | Result | Rejected statically. |
| Result or Option | An ordinary value-returning function, procedure, or program body | Rejected statically. |
| An ordinary value | Any context | Rejected: try requires Result or Option. |

Aliases preserve compatibility. Distinct error types retain their nominal identity;
they cannot propagate into their base type. Generic error parameters must preserve
the same type through the enclosing return signature, including inside aggregates.
An unconstrained generic E cannot propagate into string, or into another generic F.
Numeric error types receive no implicit conversion.

The operand evaluates once. On success the expression has the operand's payload
type; using that payload still requires ordinary type compatibility. Error or None
exits immediately, applying the function's normal cleanup and postcondition rules.

`try` handles returned Error and None values. It does not catch `panic`, failed
`Unwrap`, or other runtime failures, and does not convert them into wrapper values.
Use `is` for a nonbinding variant test, `case` for local handling or payload binding,
and `Unwrap` for a deliberate runtime failure.

## See also

- [Result](result.md)
- [Option](option.md)
- [Operators — `try` precedence](../basics/operators.md#operator-precedence)
