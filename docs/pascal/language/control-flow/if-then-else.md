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

## If expressions

In a required value position, each branch contains one expression and `else` is
required:

```pascal
const Sign: string := if X > 0 then 'positive'
  elsif X = 0 then 'zero'
  else 'negative'
end if;
```

Conditions must be Boolean. They run in written order until one is true, and
only the selected value runs. Each branch has exactly one expression, with no
statement terminator before `elsif` or `else`. The surrounding declaration or
statement supplies its terminator after `end if`.

An expected type constrains every value. Without one, all branches must agree on
one ordinary type; there is no automatic union type or integer-to-real branch
conversion. Empty collections and payloadless generic constructors need enough
context from an annotation or the other branches. A procedure call produces no
value and cannot be used as a branch; a procedure value can be selected.

In statement position, `if` keeps the statement-list syntax above.

## See also

- [Case of intro](case-of-intro.md)
