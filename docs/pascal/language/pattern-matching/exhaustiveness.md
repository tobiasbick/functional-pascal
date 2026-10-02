# Exhaustiveness

The compiler checks that `case` statements on enum, `Result`, and `Option` types cover all variants. A missing variant causes a compile-time error unless an `else` branch is present.

## Enum exhaustiveness

```pascal
type Light = enum
  Red;
  Yellow;
  Green;
end enum;

// ERROR: non-exhaustive match — missing Light.Yellow
  case L of
    when Light.Red:
      Console.WriteLn('Stop');
    when Light.Green:
      Console.WriteLn('Go');
  end case;
```

Fix by covering all variants:

```pascal
case L of
  when Light.Red:
    Console.WriteLn('Stop');
  when Light.Yellow:
    Console.WriteLn('Caution');
  when Light.Green:
    Console.WriteLn('Go');
end case;
```

Or by adding `else`:

```pascal
case L of
  when Light.Red:
    Console.WriteLn('Stop');
  else
    Console.WriteLn('Proceed with caution');
end case;
```

## Result and Option exhaustiveness

`Result` requires both `Ok` and `Error`. `Option` requires both `Some` and `None`:

```pascal
// ERROR: non-exhaustive — missing Error
case R of
  when Ok(V):
    Console.WriteLn(Conv.IntToStr(V));
end case;

// OK: both variants covered
case R of
  when Ok(V):
    Console.WriteLn(Conv.IntToStr(V));
  when Error(E):
    Console.WriteLn('Error: ' + E);
end case;
```

## Rules

- Enum types: every variant name must appear on an **unguarded** arm, or `else` must be present. Data-carrying variants count by name (`Shape.Circle`, not by field values).
- `Result`: both `Ok` and `Error` must appear on unguarded arms, or `else` must be present.
- `Option`: both `Some` and `None` must appear on unguarded arms, or `else` must be present.
- Scalar types (`integer`, `string`, `boolean`): `else` is recommended but not required.
- Guard clauses do not count toward exhaustiveness — `Shape.Circle(R) if R > 0` does not cover variant `Circle`; add an unguarded `Shape.Circle(R)` arm or `else`.

## See also

- [Enum patterns](enum-patterns.md)
- [Result and Option patterns](result-option-patterns.md)
- [Guards](guards.md)
