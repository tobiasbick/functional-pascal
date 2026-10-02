# If / elsif / else

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`if_stmt`).

The examples use `uses Std.Console as Console;`. Branches contain statement
lists; every statement ends with `;`, including the statement before `elsif` or
`else`.

```pascal
if X > 0 then
  Console.WriteLn('positive');
elsif X = 0 then
  Console.WriteLn('zero');
else
  Console.WriteLn('negative');
end if;
```

Each branch has its own local declaration scope. A binding in one branch is not
visible in a neighboring branch or after `end if;`. Use `null;` for a branch that
performs no action. An explicit `begin ... end;` creates another nested scope:

```pascal
if X > 10 then
  begin
    Console.WriteLn('large');
    X := X - 10;
  end;
else
  begin
    Console.WriteLn('small');
  end;
end if;
```

`else if` starts a nested conditional, so it needs a second `end if;`. Use `elsif`
to continue the current conditional with a single closer.

## See also

- [Case of intro](case-of-intro.md)
