# Record update

The `with` expression creates a copy of a record with selected fields replaced. The original value is never mutated.

Formal syntax: [`grammar.ebnf`](../../../specs/grammar.ebnf) (`with_expr`).

```
base with Field := NewValue; … end
```

```pascal
type
  Point = record X: integer; Y: integer; end;

var
  P: Point := record X := 1; Y := 2; end;
  Q: Point := P with X := 99; end;   { Q.X=99, Q.Y=2; P is unchanged }
```

Multiple fields can be updated in one expression:

```pascal
var
  R: Point := P with X := 10; Y := 20; end;
```

Updates may be chained by wrapping the inner expression in parentheses:

```pascal
var
  S: Point := (P with X := 5; end) with Y := 7; end;
```

`with` works on any record value, including function return values:

```pascal
function Origin(): Point;
begin
  return record X := 0; Y := 0; end
end;

var
  T: Point := Origin() with X := 42; end;
```

Unknown field names and type mismatches in override values are compile-time errors.
Each field may be overridden at most once per `with` expression. Field names
are case-insensitive, so `X` and `x` count as duplicate overrides.

## See also

- [Records](records.md)
