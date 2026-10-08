# Exhaustiveness

The compiler checks that `case` statements on enum, `Result`, and `Option` types cover all variants, including nested payload patterns. A missing variant causes a compile-time error unless an `else` branch is present.

## Enum exhaustiveness

```pascal
type
  Light = enum
    Red;
    Yellow;
    Green;
  end enum;

// ERROR: non-exhaustive match — missing Light.Yellow

case L of
  when Light.Red:
    WriteLn('Stop');
  when Light.Green:
    WriteLn('Go');
end case;
```

Fix by covering all variants:

```pascal
case L of
  when Light.Red:
    WriteLn('Stop');
  when Light.Yellow:
    WriteLn('Caution');
  when Light.Green:
    WriteLn('Go');
end case;
```

Or by adding `else`:

```pascal
case L of
  when Light.Red:
    WriteLn('Stop');
  else
    WriteLn('Proceed with caution');
end case;
```

## Result and Option exhaustiveness

`Result` requires both `Ok` and `Error`. `Option` requires both `Some` and `None`:

```pascal
// ERROR: non-exhaustive — missing Error
case R of
  when Ok(const V):
    WriteLn(IntToStr(V));
end case;

// OK: both variants covered
case R of
  when Ok(const V):
    WriteLn(IntToStr(V));
  when Error(const E):
    WriteLn('Error: ' + E);
end case;
```

## Nested coverage

Coverage is checked recursively. Every value of the case type must match some
unguarded arm, including the payloads of nested patterns:

```pascal
// ERROR: non-exhaustive — missing Ok(None)
case Response of
  when Ok(Some(const User)):
    Greet(User);
  when Error(_):
    Retry();
end case;
```

The diagnostic names one missing pattern per uncovered variant, such as
`Ok(None)` or `Some(Shape.Rect(_, _))`.

## Unreachable labels

A label of an enum, `Result`, or `Option` case is unreachable when earlier
unguarded arms already match every value it matches. It is an error (FP3033):

```pascal
case Item of
  when Some(_):
    Use();
  when Some(0): // ERROR: unreachable after Some(_)
    Skip();
  when None:
    null;
end case;
```

Guarded arms never make later labels unreachable.

Repeated comparisons are also unreachable: a second unguarded `Some(1)` or
`Error('timeout')` matches no new value. Named compile-time constants and
constant expressions are compared by their evaluated values, so `Some(Limit)`
and `Some(1 + 2)` overlap completely when `Limit` is `3`.

## Rules

- Enum, `Result`, and `Option` types: every value must match an **unguarded** arm, or `else` must be present. This includes nested payloads; `when Ok(Some(_)):` does not cover `Ok(None)`.
- `_` and `const Name` cover every value of their position. Enum members, `true`/`false`, and nested variants cover their own values. Named compile-time boolean and simple-enum constants contribute the same coverage as their values, including when parenthesized. Literals and constants of other types (integers, strings) never complete coverage on their own.
- Scalar types (`integer`, `string`, `boolean`): `else` is recommended but not required.
- Guard clauses do not count toward exhaustiveness — `when Shape.Circle(const R) if R > 0:` does not cover variant `Circle`; add an unguarded `when Shape.Circle(const R):` arm or `else`.
- Labels already covered by earlier unguarded arms are rejected (FP3033).

## See also

- [Enum patterns](enum-patterns.md)
- [Result and Option patterns](result-option-patterns.md)
- [Guards](guards.md)
