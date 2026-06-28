# Case syntax

```text
case <expression> of
  <label> { , <label> } [ if <boolean> ] : <statement> ;
  ...
[ else
    <statement> { ; <statement> } [ ; ] ]
end
```

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`case_stmt`, `case_label`).

- Separate arms with `;`. A trailing `;` before `else` or `end` is optional.
- Do not put `;` immediately before `else` or `end` (semicolons are separators, not terminators).
- Each arm has one statement; use `begin … end` for multiple statements (see [Scalar labels — block arms](scalar-labels.md#block-arms)).

## Example

```pascal
case Status of
  0: WriteLn('ok');
  1, 2: WriteLn('retry');
else
  WriteLn('failed')
end
```

## See also

- [Scalar labels](scalar-labels.md)
