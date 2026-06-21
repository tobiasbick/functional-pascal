# Case of intro

The `case` expression may have any of the following types:

- an ordinal type: `integer`, `boolean`, `string`, or an `enum`
- `string`
- `Result of T, E` or `Option of T`

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`case_stmt`, `case_label`).

Simple scalar matching (integers, chars, strings, booleans, simple enums) is shown below. Guard clauses (`label if cond:`), destructuring patterns (`Ok(x)`, `Error(e)`, `Some(x)`, `None`), data-carrying enum patterns, and exhaustiveness rules are documented in [Pattern matching](../pattern-matching/README.md) and [Error handling](../error-handling/README.md).

```pascal
case Day of
  'Monday':    WriteLn('Start of week');
  'Friday':    WriteLn('Almost weekend');
  'Saturday',
  'Sunday':    WriteLn('Weekend');
else
  WriteLn('Midweek');
end;
```

With ranges:

```pascal
case Score of
  0..59:    Grade := 'F';
  60..69:   Grade := 'D';
  70..79:   Grade := 'C';
  80..89:   Grade := 'B';
  90..100:  Grade := 'A';
end;
```

## See also

- [Pattern matching](../pattern-matching/README.md)
- [Error handling](../error-handling/README.md)
