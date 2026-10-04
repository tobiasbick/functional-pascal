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

Payload patterns match fields positionally. Use `const` to introduce an immutable
arm-local binding, `_` to ignore a payload, or a literal/static constant to compare:

```pascal
uses Std.Console as Console;

case S of
  when Shape.Circle(const R):
    Console.WriteLn('Circle');
  when Shape.Rectangle(const W, const H):
    Console.WriteLn('Rectangle');
  when Shape.Point:
    Console.WriteLn('Point');
end case;
```

Rules:

- A plain identifier refers to a static constant; it never introduces a binding.
- Duplicate binding names in one pattern are errors. Grouped labels must bind
  the same names with the same types. Bindings are available to the guard and
  body, and cannot be reassigned or redeclared in the same arm scope.
- A binding may shadow an outer local, but cannot shadow an import alias.
- A pattern variant must belong to the scrutinee enum type (`Shape.Circle` when matching `Shape`).
- The complete variant name is resolved. A matching final member name from another
  enum does not match, and unknown qualifiers are errors. Imported patterns use
  the declared import alias. Concrete generic aliases must have the same type
  arguments as the scrutinee; a generic declaration takes its payload types from
  the expected scrutinee type.
- Use an `if` guard for additional constraints on a bound value. The guard runs
  after the entire pattern matches.

## Nested patterns

Patterns can inspect nested enum, Option and Result payloads:

```pascal
type Response of (T) = enum
  Received(Value: Option of (T));
  Missing;
end enum;

function Extract(ResponseValue: Response of (integer)): integer;
begin
  return case ResponseValue of
    when Response.Received(Option.Some(const Value)): Value;
    when Response.Received(Option.None): 0;
    when Response.Missing: 0;
  end case;
end function;
```

The outer variant is checked before its payload is read. Nested variants use
qualified names, including `Option.Some`, `Option.None`, `Result.Ok` and
`Result.Error`. Fieldless variants are values: write `Response.Missing`, without
`()`.

Enum cases cover every variant explicitly. An arm for
`Response.Received(Option.Some(42))` covers only that payload; add the other
payload alternatives or `Response.Received(_)`. Top-level `_` and `else` are
rejected for closed enum, Option and Result cases. See [Exhaustiveness](exhaustiveness.md).

## See also

- [Types — enums](../types/enums.md)
- [Guards](guards.md)
- [Exhaustiveness](exhaustiveness.md)
