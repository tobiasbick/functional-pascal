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

## Pattern bindings

Patterns bind names explicitly with `const Name` and ignore a field with `_`:

```pascal
case Response of
  when Ok(const Value):
    Use(Value);
  when Error(_):
    Retry();
end case;
```

- `const Name` binds one payload field; the name is visible in the guard and
  the arm body only.
- `_` ignores exactly one field. It cannot stand for a whole variant.
- A plain identifier in a payload position compares with the compile-time
  constant or enum member it names. An unknown name reports FP3031 with a hint
  showing `const Name`. Only the explicit `const Name` form binds a value.
- Bindings are read-only; there is no `var` binding.
- A scalar `case` binds the matched value with `when const Name if Guard:`
  (see [Guards](guards.md#scalar-guard-bindings)). A bare identifier label is
  always a value comparison.

## Nested patterns and comparisons

Patterns nest inside payload positions, and a payload may compare with a
literal, a named compile-time constant, or an enum member:

```pascal
case Response of
  when Ok(Some(const User)):
    Greet(User);
  when Ok(None):
    null;
  when Error('timeout'):
    Retry();
  when Error(const Message):
    Report(Message);
end case;
```

- Comparisons apply to ordinal, enum, and string values. Bind other values
  with `const Name` and test them in a guard.
- A bare identifier in a payload position compares with the constant it
  names. Computed `const` values belong in a guard (FP3014); a name that
  resolves to nothing reports FP3031 with the `const Name` form.
- Nested Result, Option, and enum patterns follow the same rules as top-level
  labels. Arms are tested in source order; the first matching arm whose guard
  holds runs.
- Comparison names resolve in the surrounding scope before the arm's bindings
  are introduced. In `Pair.Both(const Limit, Limit)`, the first field binds a
  new `Limit` for the guard and body; the second compares with the surrounding
  compile-time constant `Limit`.

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
- [Enum patterns](enum-patterns.md)
- [Result and Option patterns](result-option-patterns.md)
