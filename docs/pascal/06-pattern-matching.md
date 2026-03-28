# 6. Case Of

The `case of` statement matches a value against constants, ranges, or enum variants.

## Basic Matching

```pascal
case Value of
  1: Std.Console.WriteLn('one');
  2: Std.Console.WriteLn('two');
  3: Std.Console.WriteLn('three');
else
  Std.Console.WriteLn('other');
end;
```

## Multiple Values

Separate multiple values with commas:

```pascal
case Day of
  'Monday':    Std.Console.WriteLn('Start of week');
  'Friday':    Std.Console.WriteLn('Almost weekend');
  'Saturday',
  'Sunday':    Std.Console.WriteLn('Weekend');
else
  Std.Console.WriteLn('Midweek');
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
  Light.Red:  Std.Console.WriteLn('Stop');
else
  Std.Console.WriteLn('Proceed with caution');
end;
```

## Block Arms

Use `begin..end` when a case arm needs multiple statements:

```pascal
case Command of
  'help':
    begin
      Std.Console.WriteLn('Available commands:');
      Std.Console.WriteLn('  help, quit, run');
    end;
  'quit':
    Std.Console.WriteLn('Goodbye');
else
  Std.Console.WriteLn('Unknown command');
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
    Std.Console.WriteLn('Large circle');
  Shape.Circle(R):
    Std.Console.WriteLn('Small circle');
  Shape.Rectangle(W, H) if W = H:
    Std.Console.WriteLn('Square');
  Shape.Rectangle(W, H):
    Std.Console.WriteLn('Rectangle');
  Shape.Point:
    Std.Console.WriteLn('Point');
end;
```

The guard expression has access to any bindings introduced by the label.

## Nested Patterns

Patterns can be nested to match complex enum structures. When a variant field is itself an enum, the pattern can destructure it recursively:

```pascal
type
  Expr = enum
    Num(Value: integer);
    Add(Left: Expr; Right: Expr);
    Mul(Left: Expr; Right: Expr);
  end;

case E of
  Expr.Add(Expr.Num(A), Expr.Num(B)):
    Std.Console.WriteLn('Simple addition: ' + Std.Conv.IntToStr(A) + ' + ' + Std.Conv.IntToStr(B));
  Expr.Mul(Expr.Num(0), _):
    Std.Console.WriteLn('Multiply by zero');
else
  Std.Console.WriteLn('Complex expression');
end;
```

### Wildcards

Use `_` to match any value without binding it:

```pascal
case E of
  Expr.Mul(Expr.Num(0), _):
    Std.Console.WriteLn('Multiply by zero');
  Expr.Mul(_, Expr.Num(0)):
    Std.Console.WriteLn('Multiply by zero');
else
  Std.Console.WriteLn('Other');
end;
```

### Guards with Nested Patterns

Guard clauses have access to all bindings from nested patterns:

```pascal
case E of
  Expr.Add(Expr.Num(A), Expr.Num(B)) if A + B > 100:
    Std.Console.WriteLn('Large sum');
  Expr.Add(Expr.Num(A), Expr.Num(B)):
    Std.Console.WriteLn('Small sum');
else
  Std.Console.WriteLn('Not a simple addition');
end;
```

### Non-recursive Nesting

Nested patterns also work between different enum types:

```pascal
type
  Inner = enum
    A(X: integer);
    B;
  end;
  Outer = enum
    Wrap(I: Inner);
    Empty;
  end;

case V of
  Outer.Wrap(Inner.A(X)):
    Std.Console.WriteLn('A: ' + Std.Conv.IntToStr(X));
  Outer.Wrap(Inner.B):
    Std.Console.WriteLn('B');
  Outer.Empty:
    Std.Console.WriteLn('Empty');
end;
```

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
  Light.Red:   Std.Console.WriteLn('Stop');
  Light.Green: Std.Console.WriteLn('Go');
end;
```

Fix by covering all variants:

```pascal
case L of
  Light.Red:    Std.Console.WriteLn('Stop');
  Light.Yellow: Std.Console.WriteLn('Caution');
  Light.Green:  Std.Console.WriteLn('Go');
end;
```

Or by adding `else`:

```pascal
case L of
  Light.Red: Std.Console.WriteLn('Stop');
else
  Std.Console.WriteLn('Proceed with caution');
end;
```

### Result and Option Exhaustiveness

`Result` requires both `Ok` and `Error`. `Option` requires both `Some` and `None`:

```pascal
{ ERROR: non-exhaustive — missing Error }
case R of
  Ok(V): Std.Console.WriteLn(Std.Conv.IntToStr(V));
end;

{ OK: both variants covered }
case R of
  Ok(V):  Std.Console.WriteLn(Std.Conv.IntToStr(V));
  Error(E): Std.Console.WriteLn('Error: ' + E);
end;
```

### Rules

- Enum types: all variants must be covered or `else` present.
- `Result`: both `Ok` and `Error` must be handled or `else` present.
- `Option`: both `Some` and `None` must be handled or `else` present.
- Scalar types (`integer`, `string`, `char`, `boolean`): `else` is recommended but not required.
- Guard clauses do not count toward exhaustiveness — `Shape.Circle(R) if R > 0` does not cover `Shape.Circle`.
