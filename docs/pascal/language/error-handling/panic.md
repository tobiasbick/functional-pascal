# Panic

Use `panic` to abort the program when an unrecoverable error occurs:

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`panic_stmt`).

```pascal
begin
  panic('Something went terribly wrong');
end.
```

## Guarding assumptions

```pascal
function DivideChecked(A: integer; B: integer): integer;
begin
  if B = 0 then
    panic('Division by zero');
  return A div B
end;
```

## When to use panic vs Result

| Use | When |
|-----|------|
| `Result` / `Option` | Expected failure conditions (user input, file not found, search miss) |
| `panic` | Programming logic errors, broken invariants, impossible cases |

## See also

- [Result](result.md)
- [Option](option.md)
