# Case of intro

The `case` expression may have any of the following types:

- an ordinal type: `integer`, `boolean`, or an `enum`
- `string`
- `Result of (T, E)` or `Option of (T)`

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`case_stmt`, `case_label`).

Simple scalar matching (integers, strings including single-character strings, booleans, and simple enums) is shown below. Guard clauses (`label if cond:`), destructuring patterns (`Result.Ok(x)`, `Result.Error(e)`, `Option.Some(x)`, `Option.None`), data-carrying enum patterns, and exhaustiveness rules are documented in [Pattern matching](../pattern-matching/README.md) and [Error handling](../error-handling/README.md).

```pascal
uses Std.Console as Console;

case Day of
  when 'Monday':
    Console.WriteLn('Start of week');
  when 'Friday':
    Console.WriteLn('Almost weekend');
  when 'Saturday', 'Sunday':
    Console.WriteLn('Weekend');
  else
    Console.WriteLn('Midweek');
end case;
```

With ranges:

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

## See also

- [Pattern matching](../pattern-matching/README.md)
- [Error handling](../error-handling/README.md)
