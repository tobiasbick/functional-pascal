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

Enum patterns bind variant fields positionally with plain identifiers only:

```pascal
case S of
  when Shape.Circle(R):
    WriteLn('Circle');
  when Shape.Rectangle(W, H):
    WriteLn('Rectangle');
  when Shape.Point:
    WriteLn('Point');
end case;
```

Rules:

- Each field position is a bare identifier binding, matched positionally to the variant fields.
- A pattern variant must belong to the scrutinee enum type (`Shape.Circle` when matching `Shape`).
- Use an `if` guard for additional constraints on a bound value (literals, ranges, or comparisons).

## See also

- [Types — enums](../types/enums.md)
- [Guards](guards.md)
- [Exhaustiveness](exhaustiveness.md)
