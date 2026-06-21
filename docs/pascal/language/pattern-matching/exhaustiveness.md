# Exhaustiveness

The compiler checks that `case` statements on enum, `Result`, and `Option` types cover all variants. A missing variant causes a compile-time error unless an `else` branch is present.

## Enum exhaustiveness

```pascal
type
  Light = enum
    Red;
    Yellow;
    Green;
  end;

{ ERROR: non-exhaustive match — missing Light.Yellow }
case L of
  Light.Red:   WriteLn('Stop');
  Light.Green: WriteLn('Go');
end;
```

Fix by covering all variants:

```pascal
case L of
  Light.Red:    WriteLn('Stop');
  Light.Yellow: WriteLn('Caution');
  Light.Green:  WriteLn('Go');
end;
```

Or by adding `else`:

```pascal
case L of
  Light.Red: WriteLn('Stop');
else
  WriteLn('Proceed with caution');
end;
```

## Result and Option exhaustiveness

`Result` requires both `Ok` and `Error`. `Option` requires both `Some` and `None`:

```pascal
{ ERROR: non-exhaustive — missing Error }
case R of
  Ok(V): WriteLn(IntToStr(V));
end;

{ OK: both variants covered }
case R of
  Ok(V):  WriteLn(IntToStr(V));
  Error(E): WriteLn('Error: ' + E);
end;
```

## Rules

- Enum types: every variant name must appear on an **unguarded** arm, or `else` must be present. Data-carrying variants count by name (`Shape.Circle`, not by field values).
- `Result`: both `Ok` and `Error` must appear on unguarded arms, or `else` must be present.
- `Option`: both `Some` and `None` must appear on unguarded arms, or `else` must be present.
- Scalar types (`integer`, `string`, `string`, `boolean`): `else` is recommended but not required.
- Guard clauses do not count toward exhaustiveness — `Shape.Circle(R) if R > 0` does not cover variant `Circle`; add an unguarded `Shape.Circle(R)` arm or `else`.

## See also

- [Enum patterns](enum-patterns.md)
- [Result and Option patterns](result-option-patterns.md)
- [Guards](guards.md)
