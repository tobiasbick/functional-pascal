# While and repeat

The code fragments use `uses Std.Proc as Proc, Std.Console as Console;`.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`while_stmt`, `repeat_stmt`).

## While loop

The body contains a nonempty list of statements and closes with `end while;`.
Every body statement ends with `;`. Body-local declarations are unavailable
in the condition and after the loop; the condition uses the enclosing scope.

```pascal id=example-1 fragment context=body-console
var Count: integer := 0;
while Count < 10 do
  Console.WriteLn(Count);
  Count := Count + 1;
end while;
```

A loop condition may use a nonbinding `is` test. To process a payload,
use `case` inside the loop; the selector is evaluated once per iteration:

```pascal id=example-2 fragment context=queue-loop
while true do
  case Queue.Next() of
    when Some(const Job):
      discard Proc.Run(Job, []);
    when None:
      break;
  end case;
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

```pascal id=example-3 fragment context=body-console
var Input: option of string := None;
repeat
  Input := Console.ReadLn();
until (Input is None) or (Input is Some('quit'));
```

Both loop forms require an explicit `null;` when their body has no action.
For example, `repeat null; until Ready();` retains the condition's repeated
evaluation without an empty statement list.

## See also

- [Break and continue](break-continue.md)
