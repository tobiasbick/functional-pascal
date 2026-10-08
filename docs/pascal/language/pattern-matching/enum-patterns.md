# Enum patterns

Match on enum variants:

```pascal
type Direction = enum
  North;
  South;
  East;
  West;
end enum;

function DirectionName(D: Direction): string;
begin
  case D of
    when Direction.North:
      return 'North';
    when Direction.South:
      return 'South';
    when Direction.East:
      return 'East';
    when Direction.West:
      return 'West';
  end case;
end function;
```

Enum patterns bind variant fields positionally with `const Name`, or ignore them with `_`:

```pascal
case S of
  when Shape.Circle(const R):
    WriteLn('Circle');
  when Shape.Rectangle(const W, const H):
    WriteLn('Rectangle');
  when Shape.Point:
    WriteLn('Point');
end case;
```

Rules:

- Each field position holds a pattern: `const Name` binds the field, `_` ignores it, a literal or constant compares with it, and a nested variant or Result/Option pattern matches its structure, for example `Shape.Tinted(Color.Red, const Size)`. Fields match positionally; binding names need not match field names. A plain identifier that names no constant, such as `Shape.Circle(R)`, is rejected (FP3031). Named fields such as `when Shape.Circle(Radius := const R):` are rejected (FP3026), although [construction](../types/enums.md) accepts them.
- A binding name may appear only once per pattern.
- A pattern variant must belong to the scrutinee enum type (`Shape.Circle` when matching `Shape`). The complete name follows ordinary name resolution, including unit and type aliases. An unknown qualifier is an error; a variant from another enum cannot match merely because it has the same short name.
- Use an `if` guard for constraints a pattern cannot express, such as ranges or comparisons with computed values.

## See also

- [Types — enums](../types/enums.md)
- [Guards](guards.md)
- [Exhaustiveness](exhaustiveness.md)
