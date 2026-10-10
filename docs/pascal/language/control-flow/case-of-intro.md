# Case of intro

A `case` statement selects the first arm whose label and optional guard match.
The expression is evaluated once. Supported expression types are:

- ordinal types: `integer`, `boolean`, or an `enum`
- `string`
- `Result of T, E` or `Option of T`
- a [distinct type](../types/distinct-types.md#case) over `integer`, `string`,
  or `boolean`, with labels of the same distinct type

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`case_stmt`, `case_arm`, `case_label`).

Each arm starts with `when Labels [if Guard]:` and contains a nonempty list of
statements. The next `when`, optional final `else`, or `end case` ends the list.
Every statement ends with `;`, including the last statement in each arm; the
whole case ends with `end case;`. Use `null;` for an arm with no action.
A case requires at least one `when` arm. Arms do not fall through.

```pascal
case Day of
  when 'Monday':
    WriteLn('Start of week');
  when 'Friday':
    WriteLn('Almost weekend');
  when 'Saturday', 'Sunday':
    WriteLn('Weekend');
  else
    WriteLn('Midweek');
end case;
```

Ranges use `..`; comma-separated labels share one body:

```pascal
case Score of
  when 0..59:
    Grade := 'F';
  when 60..69:
    Grade := 'D';
  when 70..79:
    Grade := 'C';
  when 80..89:
    Grade := 'B';
  when 90..100:
    Grade := 'A';
end case;
```

## Arm scopes and guards

Each arm, including `else`, has its own local scope. Body declarations cannot
be used in its guard, another arm, or after the case. Pattern bindings are
available in the guard and selected body. An explicit `begin ... end;` inside
an arm retains an additional scope; its ending closes only that inner block.

```pascal
case Items of
  when Some(const Value) if Value > 0:
    const Doubled: integer := Value * 2;
    WriteLn(Doubled);
    WriteLn('positive');
  when Some(const Value):
    null;
  when None:
    WriteLn('missing');
end case;
```

Guarded arms do not count toward exhaustive variant coverage. The compiler
requires complete explicit enum, `Result`, and `Option` coverage. These types
reject `else` (FP3035), even after complete coverage. Scalar `integer`, `string`,
and `boolean` cases allow `else`; Boolean coverage is optional. Use an
[`is` test](../pattern-matching/is-test.md) when handling just one variant.
See [Pattern matching](../pattern-matching/README.md) for guards, scalar guard
bindings, data-enum patterns, and exhaustiveness rules. Destructuring keeps
forms such as `Ok(const Value)`, `Error(const Reason)`, `Some(const Value)`,
and `None`.

## See also

- [Pattern matching](../pattern-matching/README.md)
- [Error handling](../error-handling/README.md)
