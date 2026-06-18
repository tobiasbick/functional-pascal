# Ranges

Use `..` to match a range of values:

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`case_label` — range form).

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

- [Scalar labels](scalar-labels.md)
- [Guards](guards.md)
