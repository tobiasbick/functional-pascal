# Exhaustiveness

Statement and expression cases use the same recursive pattern and coverage rules.
Closed enums, `Option`, and `Result` require explicit coverage of every variant
and its payload alternatives. They reject `else` and top-level `_` or `const`
bindings. A payload `_` can cover the remaining values of a named variant.

## Closed variants

```pascal
type Light = enum
  Red;
  Yellow;
  Green;
end enum;

function Direction(L: Light): string;
begin
  return case L of
    when Light.Red: 'stop';
    when Light.Yellow: 'caution';
    when Light.Green: 'go';
  end case;
end function;
```

Removing `Light.Yellow` produces a non-exhaustive-case error. Cover that variant
explicitly, including when several variants share one arm:

```pascal
case L of
  when Light.Red:
    Console.WriteLn('Stop');
  when Light.Yellow, Light.Green:
    Console.WriteLn('Proceed with caution');
end case;
```

`Option` requires `Option.Some` and `Option.None`; `Result` requires `Result.Ok`
and `Result.Error`. Coverage examines nested payloads, rather than counting only
the outer variant names:

```pascal
type Choice = enum
  Present(Value: Option of (boolean));
  Missing;
end enum;

function Describe(Value: Choice): string;
begin
  return case Value of
    when Choice.Present(Option.Some(true)): 'true';
    when Choice.Present(Option.Some(false)): 'false';
    when Choice.Present(Option.None): 'none';
    when Choice.Missing: 'missing';
  end case;
end function;
```

`Choice.Present(_)` can replace all three `Present` arms. Matching only
`Choice.Present(Option.Some(const Value))` leaves `Option.None` uncovered.

## Guards and unreachable patterns

Guards do not establish coverage: even a guard written as `if true` is a guard.
After a guarded variant arm, add an unguarded arm for its remaining payloads.
Patterns in one grouped arm must introduce the same binding names and types.

Duplicate or unreachable patterns are errors. For example, an unguarded
`Choice.Present(_)` makes a later `Choice.Present(Option.None)` unreachable.
A failed guard continues with the next arm. The scrutinee is evaluated once.

## Scalar cases

An integer or string statement case may omit `else` and do nothing on no match.
A case expression must produce a value, so it requires `else` unless finite
coverage is proven:

```pascal
const Description: string := case Number of
  when 0: 'zero';
  when 1..9: 'single digit';
  else 'other';
end case;
```

Both Boolean labels prove finite coverage. An `else` after complete finite
coverage is unreachable and rejected.

## See also

- [Enum patterns](enum-patterns.md)
- [Result and Option patterns](result-option-patterns.md)
- [Guards](guards.md)
