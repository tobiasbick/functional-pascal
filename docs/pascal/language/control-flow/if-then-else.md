# If / then / else

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`if_stmt`).

Terminate each branch with `;`, including the branch before `else`. With a
single-statement body, the last branch's terminator also ends the `if`
statement. `else` binds to the nearest unmatched `if`; use an explicit block
to make a branch containing a nested conditional unambiguous.

```pascal
if X > 0 then
  WriteLn('positive');
else if X = 0 then
  WriteLn('zero');
else
  WriteLn('negative');
```

With blocks:

```pascal
if X > 10 then
begin
  WriteLn('large');
  X := X - 10;
end;
else
begin
  WriteLn('small');
end;
```

## See also

- [Case of intro](case-of-intro.md)
