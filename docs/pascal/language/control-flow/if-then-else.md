# If / then / else

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`if_stmt`).

Each branch contains one or more statements. Every statement ends with `;`,
including the last before `elsif`, `else`, or `end if;`. The named ending
closes the whole conditional. Conditions are evaluated in order, and only
the first matching branch runs. An `else` branch is optional.

```pascal
if X > 0 then
  WriteLn('positive');
elsif X = 0 then
  WriteLn('zero');
else
  WriteLn('negative');
end if;
```

Multiple statements need no compound wrapper:

```pascal
if X > 10 then
  WriteLn('large');
  X := X - 10;
else
  WriteLn('small');
end if;
```

Every branch has its own local scope. Its declarations are unavailable in
other branches, later `elsif` conditions, or after the conditional. A plain
`begin ... end;` inside a branch adds a nested scope.

`elsif` continues the existing conditional. `else if` starts a nested
conditional and requires two matching endings:

```pascal
if Outer then
  null;
else
  if Inner then
    null;
  end if;
end if;
```

A condition may test a pattern and bind its names for the branch, for example
`if Item is Some(const Value) and Value > 0 then`; see
[Pattern test with `is`](../pattern-matching/is-test.md).

An empty branch is an error; write `null;` to state that no action is needed.
The formatter preserves nested conditionals and explicit scoping blocks.

## See also

- [Case of intro](case-of-intro.md)
