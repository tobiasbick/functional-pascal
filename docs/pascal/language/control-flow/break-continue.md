# Break and continue

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`break_stmt`, `continue_stmt`).

```pascal
for I: integer := 1 to 100 do
  begin
    if I mod 2 = 0 then
      continue;
    end if;

    if I > 50 then
      break;
    end if;

    WriteLn(I);
  end;
end for;
```

## See also

- [For loops](for-loops.md)
- [For-in](for-in.md)
