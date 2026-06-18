# For loops

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`for_stmt`).

## Counting up

```pascal
for I: integer := 1 to 10 do
begin
  WriteLn(I);
end;
```

## Counting down

```pascal
for I: integer := 10 downto 1 do
begin
  WriteLn(I);
end;
```

## See also

- [For-in](for-in.md)
- [Break and continue](break-continue.md)
