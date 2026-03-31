# 6. Case Of

The `case of` statement matches a value against constants, ranges, or enum variants.

## Basic Matching

```pascal
case Value of
  1: WriteLn('one');
  2: WriteLn('two');
  3: WriteLn('three');
else
  WriteLn('other');
end;
```

## Multiple Values

Separate multiple values with commas:

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

## Ranges

Use `..` to match a range of values:

```pascal
case Score of
  0..59:    Grade := 'F';
  60..69:   Grade := 'D';
  70..79:   Grade := 'C';
  80..89:   Grade := 'B';
  90..100:  Grade := 'A';
end;
```

## Enum Matching

Match on enum variants:

```pascal
type
  Direction = enum
    North;
    South;
    East;
    West;
  end;

function DirectionName(D: Direction): string;
begin
  case D of
    Direction.North: return 'North';
    Direction.South: return 'South';
    Direction.East:  return 'East';
    Direction.West:  return 'West';
  end;
end;
```

## Else Branch

Use `else` to handle all remaining cases:

```pascal
case L of
  Light.Red:  WriteLn('Stop');
else
  WriteLn('Proceed with caution');
end;
```

## Block Arms

Use `begin..end` when a case arm needs multiple statements:

```pascal
case Command of
  'help':
    begin
      WriteLn('Available commands:');
      WriteLn('  help, quit, run');
    end;
  'quit':
    WriteLn('Goodbye');
else
  WriteLn('Unknown command');
end;
```

## Guard Clauses

Add conditions to case arms with `if`. The guard is evaluated after the label matches; the arm executes only when the guard is `true`:

```pascal
function Classify(N: integer): string;
begin
  case N of
    0:
      return 'zero';
    N if N > 0:
      return 'positive';
    N if N < 0:
      return 'negative';
  end;
end;
```

Guards work with all label types — values, ranges, destructuring, and enum patterns:

```pascal
case S of
  Shape.Circle(R) if R > 10.0:
    WriteLn('Large circle');
  Shape.Circle(R):
    WriteLn('Small circle');
  Shape.Rectangle(W, H) if W = H:
    WriteLn('Square');
  Shape.Rectangle(W, H):
    WriteLn('Rectangle');
  Shape.Point:
    WriteLn('Point');
end;
```

The guard expression has access to any bindings introduced by the label.

### Scalar Guard Bindings

In scalar `case` arms, a single bare identifier with a guard introduces a binding for the matched value:

```pascal
case Value of
  N if N > 0:
    WriteLn('positive');
  N if N < 0:
    WriteLn('negative');
else
  WriteLn('zero');
end;
```

`N` is available in both the guard and the arm body, but only inside that arm.

Rules:

- The arm must have exactly one label.
- The label must be a single bare identifier, not a range or a comma-separated label list.
- If the identifier resolves to a compile-time constant or enum member, it remains a normal value label instead of becoming a binding.

## Exhaustiveness Checking

The compiler checks that `case` statements on enum, `Result`, and `Option` types cover all variants. A missing variant causes a compile-time error unless an `else` branch is present.

### Enum Exhaustiveness

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

### Result and Option Exhaustiveness

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

### Rules

- Enum types: all variants must be covered or `else` present.
- `Result`: both `Ok` and `Error` must be handled or `else` present.
- `Option`: both `Some` and `None` must be handled or `else` present.
- Scalar types (`integer`, `string`, `char`, `boolean`): `else` is recommended but not required.
- Guard clauses do not count toward exhaustiveness — `Shape.Circle(R) if R > 0` does not cover `Shape.Circle`.
