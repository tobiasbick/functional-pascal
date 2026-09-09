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
- `fpas fmt` omits an optional `;` immediately before `else` or `end` in its
  canonical output.
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
