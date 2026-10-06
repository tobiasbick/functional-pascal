# Case syntax

```text
case <expression> of
  when <label> { , <label> } [ if <boolean> ]:
    <terminated-statement> { <terminated-statement> }
  { ... more when arms ... }
[ else
    <terminated-statement> { <terminated-statement> } ]
end case;
```

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`case_stmt`, `case_label`).

- Each `when` or `else` arm contains a nonempty statement list with its own
  scope. Every statement ends with `;`, including the last one before another
  arm or `end case`. Write `null;` for an arm with no action.
- The case statement itself ends with `end case;`. A case requires at least
  one `when` arm; the optional `else` must follow all `when` arms.
- `fpas fmt` emits every required terminator.
- Plain `begin … end;` statements retain an additional nested scope inside an
  arm (see [Scalar labels — block arms](scalar-labels.md#block-arms)).

## Example

```pascal
case Status of
  when 0:
    WriteLn('ok');
  when 1, 2:
    WriteLn('retry');
  else
    WriteLn('failed');
end case;
```

## See also

- [Scalar labels](scalar-labels.md)
