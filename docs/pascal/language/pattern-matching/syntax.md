# Case syntax

```text
case <expression> of
  <label> { , <label> } [ if <boolean> ] : <terminated-statement>
  ...
[ else
    { <terminated-statement> } ]
end;
```

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`case_stmt`, `case_label`).

- Every arm body ends with `;`, including the last arm before `else` or `end`.
- Every statement in the `else` body also ends with `;`. The case statement
  itself ends with `end;`.
- `fpas fmt` emits every required terminator.
- Each arm has one statement; use `begin … end` for multiple statements (see [Scalar labels — block arms](scalar-labels.md#block-arms)).

## Example

```pascal
case Status of
  0: WriteLn('ok');
  1, 2: WriteLn('retry');
else
  WriteLn('failed');
end;
```

## See also

- [Scalar labels](scalar-labels.md)
