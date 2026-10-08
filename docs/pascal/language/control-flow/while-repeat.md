# While and repeat

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`while_stmt`, `repeat_stmt`).

## While loop

The body contains a nonempty list of statements and closes with `end while;`.
Every body statement ends with `;`. Body-local declarations are unavailable
in the condition and after the loop; the condition uses the enclosing scope.

```pascal
var Count: integer := 0;
while Count < 10 do
  WriteLn(Count);
  Count := Count + 1;
end while;
```

A `while` condition may test a pattern; its bindings are visible in the body
and are evaluated again before every iteration:

```pascal
while Queue.Next() is Some(const Job) do
  Run(Job);
end while;
```

See [Pattern test with `is`](../pattern-matching/is-test.md).

## Repeat-until loop

The `until` condition uses the enclosing scope. Variables declared inside the
repeat body are not visible in that condition. If a body-local variable shadows
an outer name, the condition still uses the outer binding, including after
`continue`.

The body executes at least once. Every body statement ends with `;`, including
the last one before `until`; the condition also ends with `;`:

```pascal
var Input: string := '';
repeat
  Input := ReadLn();
until Input = 'quit';
```

Both loop forms require an explicit `null;` when their body has no action.
For example, `repeat null; until Ready();` retains the condition's repeated
evaluation without an empty statement list.

## See also

- [Break and continue](break-continue.md)
