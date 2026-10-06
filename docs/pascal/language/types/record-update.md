# Record update

The `with` expression creates a copy of a record with selected fields replaced. The original value is never mutated.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`record_update`).

```
base with Field := NewValue; … end with
```

At least one field assignment is required. An empty update such as
`P with end with` is invalid; use `P` directly instead. Each field assignment
requires `;`, including the last one before `end with`.

The expression has no final terminator of its own. In `Apply(P with X := 1;
end with)`, no `;` separates `end with` from `)`. A following `;` terminates
the enclosing statement or declaration.

```pascal
type
  Point = record
    X: integer;
    Y: integer;
  end record;

var
  P: Point := record
    X := 1;
    Y := 2;
  end;
  Q: Point := P with X := 99; end with; // Q.X=99, Q.Y=2; P is unchanged
```

Multiple fields can be updated in one expression:

```pascal
var
  R: Point := P with X := 10; Y := 20; end with;
```

Updates may be chained by wrapping the inner expression in parentheses:

```pascal
var
  S: Point := (P with X := 5; end with) with Y := 7; end with;
```

`with` works on any record value, including function return values:

```pascal
function Origin(): Point;
begin
  return record
    X := 0;
    Y := 0;
  end;
end function;

var
  T: Point := Origin() with X := 42; end with;
```

Unknown field names and type mismatches in override values are compile-time errors.
Each field may be overridden at most once per `with` expression. Field names
are case-insensitive, so `X` and `x` count as duplicate overrides.

## See also

- [Records](records.md)
