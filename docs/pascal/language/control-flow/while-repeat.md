# While and repeat

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`while_stmt`, `repeat_stmt`).

## While loop

The body is one terminated statement. Its final `;` also terminates the loop.
Inside a compound body, every statement ends with `;` before `end;`.

```pascal
mutable var
  Count: integer := 0;

while Count < 10 do
begin
  WriteLn(Count);
  Count := Count + 1;
end;
```

## Repeat-until loop

The `until` condition uses the enclosing scope. Variables declared inside the
repeat body are not visible in that condition. If a body-local variable shadows
an outer name, the condition still uses the outer binding, including after
`continue`.

The body executes at least once. Every body statement ends with `;`, including
the last one before `until`; the condition also ends with `;`:

```pascal
mutable var
  Input: string := '';

repeat
  Input := ReadLn();
until Input = 'quit';
```

## See also

- [Break and continue](break-continue.md)
