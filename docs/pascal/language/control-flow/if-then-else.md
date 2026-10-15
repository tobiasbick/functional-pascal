# If / then / else

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`if_stmt`, `if_expression`).

Each branch contains one or more statements. Every statement ends with `;`,
including the last before `elsif`, `else`, or `end if;`. The named ending
closes the whole conditional. Conditions are evaluated in order, and only
the first matching branch runs. An `else` branch is optional.

```pascal id=example-1 fragment context=integer-condition
if X > 0 then
  Console.WriteLn('positive');
elsif X = 0 then
  Console.WriteLn('zero');
else
  Console.WriteLn('negative');
end if;
```

Multiple statements need no compound wrapper:

```pascal id=example-2 fragment context=integer-condition
if X > 10 then
  Console.WriteLn('large');
  X := X - 10;
else
  Console.WriteLn('small');
end if;
```

Every branch has its own local scope. Its declarations are unavailable in
other branches, later `elsif` conditions, or after the conditional. A plain
`begin ... end;` inside a branch adds a nested scope.

`elsif` continues the existing conditional. `else if` starts a nested
conditional and requires two matching endings:

```pascal id=example-3 fragment context=nested-condition
if Outer then
  null;
else
  if Inner then
    null;
  end if;
end if;
```

A condition may test a pattern without binding names, for example
`if Item is Some(_) and Ready then`; see
[Pattern test with `is`](../pattern-matching/is-test.md).

An empty branch is an error; write `null;` to state that no action is needed.
The formatter preserves nested conditionals and explicit scoping blocks.

## If expressions

Where a value is expected, `if` selects one of several values:

```pascal id=example-4 fragment context=condition-expression
const Noun: string := if Count = 1 then 'item' else 'items' end if;

function Grade(Points: integer): string;
begin
  return if Points >= 90 then 'A' elsif Points >= 50 then 'B' else 'C' end if;
end function;
```

- Each branch is exactly one expression, without `;`. `else` is required, and
  `end if` closes the expression; the enclosing statement or declaration
  supplies the terminator.
- Conditions are evaluated in order, and only the selected branch value is
  evaluated.
- All branches have one type, checked like an assignment. There is no numeric
  widening: `if Ready then 1 else 2.5 end if` is an error; write `1.0` or
  `Conv.IntToReal(1)` with `uses Std.Conv as Conv;`. Values that take their type from context, such as `None` or
  `[]`, take the type of the other branches, for example
  `if Found then Some(Value) else None end if`. The result must then match the
  expected type of its position. Branches also complete result-only generic
  calls: `if Found then Empty() else Some(1) end if` determines integer for
  `function Empty<T>(): Option of T`.
- A condition may use nonbinding `is` pattern tests with the same rules as the
  statement: `if Item is Some(_) then 'present' else 'absent' end if`.
  To extract a payload, use a [case expression](case-of-intro.md#case-expressions).
- When every condition and branch is a compile-time constant, the expression is
  one too, so `const Limit: integer := if Debug then 10 else 100 end if;` can be
  used as a `case` label.

An `if` at the start of a statement is always the `if` statement. The
expression form appears after `:=`, `return`, or `discard`, as an argument, as
an operand, or inside a condition. Because `end if` closes it, it needs no
parentheses as an operand: `1 + if Ready then 2 else 3 end if`.

The formatter keeps a short `if` expression on one line. A longer one, or one
with comments, puts `elsif`, `else`, and `end if` on their own lines, indented
one level:

```pascal id=example-5 fragment context=mode-expression
const Label: string := if Mode = 1 then 'a rather long first branch value'
  elsif Mode = 2 then 'another long branch value here'
  else 'and the final else branch'
  end if;
```

## See also

- [Case of intro](case-of-intro.md)
