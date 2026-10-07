# Ranges

Use `..` to match a range of values:

Both endpoints must be [compile-time constants](../basics/constants.md#compile-time-constants).
For dynamic bounds, use a [guard](guards.md), for example
`when Value if Value >= Lower and Value <= Upper:`.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`case_label` — range form).

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

- [Scalar labels](scalar-labels.md)
- [Guards](guards.md)
