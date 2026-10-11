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
  end if;

  return A div B;
end function;
```

## When to use panic vs Result

| Use | When |
|-----|------|
| `Result` / `Option` | Expected failure conditions (user input, file not found, search miss) |
| `panic` | Programming logic errors, broken invariants, impossible cases |

An invalid dynamic [subrange conversion](../types/subranges.md) also causes a
panic. Its message includes the value and the allowed inclusive bounds; a known
invalid constant is diagnosed at compile time instead.

A failed [precondition or postcondition](../functions/contracts.md) also panics.
Its message includes the qualified routine name, original failed clause, and
current parameter values; function postconditions also include the complete
return value. Checks run on every entry or ordinary/propagated return in all
build modes, including calls through callable values and imported routines.
Completed writes through `var` parameters are preserved when a clause panics.

A panic inside a [task scope](../concurrency/task-scopes.md#failures) first cancels and
waits for the scope's children. A panic or runtime error of a scope child fails its scope: the
siblings and the scope body are cancelled, and leaving the scope re-raises the child's original
diagnostic with any further failures listed in its help text.

## See also

- [Result](result.md)
- [Option](option.md)
