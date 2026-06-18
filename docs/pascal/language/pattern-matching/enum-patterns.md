# Enum patterns

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

Enum patterns bind variant fields positionally with plain identifiers only:

```pascal
case S of
  Shape.Circle(R):
    WriteLn('Circle');
  Shape.Rectangle(W, H):
    WriteLn('Rectangle');
  Shape.Point:
    WriteLn('Point');
end;
```

Rules:

- Each field position is a bare identifier binding, matched positionally to the variant fields.
- A pattern variant must belong to the scrutinee enum type (`Shape.Circle` when matching `Shape`).
- Use a `when` guard for additional constraints on a bound value (literals, ranges, or comparisons).

## See also

- [Types — enums](../types/enums.md)
- [Guards](guards.md)
- [Exhaustiveness](exhaustiveness.md)
