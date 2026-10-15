# Enum patterns

Match on enum variants:

```pascal id=example-1 fragment context=declarations
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

Enum patterns match all fields either positionally or by their declared names.
Use `const Name` to bind a value or `_` to ignore it:

```pascal id=example-2 fragment context=shape-case
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

- Each field holds a pattern: `const Name` binds the field, `_` ignores it, a literal or constant compares with it, and a nested variant or Result/Option pattern matches its structure. `Shape.Rectangle(Height := const H, Width := const W)` is equivalent to `Shape.Rectangle(const W, const H)`. Labels are case-insensitive and may reorder fields; binding names are independent of field labels. A plain identifier naming no constant, such as `Shape.Circle(R)`, is rejected (FP3031).
- Every field is supplied exactly once. Unknown, duplicated or missing named fields report FP3025; mixed positional/named lists report FP2016. Ignore an unused field explicitly with `_`; fields have no pattern defaults or rest form. Every nested list chooses its own complete positional or named form.
- A fieldless variant uses no parentheses, for example `Shape.Point`.
- A binding name may appear only once per pattern.
- A pattern variant must belong to the scrutinee enum type (`Shape.Circle` when matching `Shape`). The complete name follows ordinary name resolution, including unit and type aliases. An unknown qualifier is an error; a variant from another enum cannot match merely because it has the same short name.
- Use an `if` guard for constraints a pattern cannot express, such as ranges or comparisons with computed values.

Generic enum patterns use the same ordinary variant names. When matching a
`Lookup of integer`, `Lookup.Found(const Value)` binds `Value` as `integer`.
Nested patterns retain the arguments at each payload position, including
recursive enums and Result or Option payloads. A variant from an alias fixed to
different arguments is rejected. All alternatives remain subject to
[exhaustiveness checking](exhaustiveness.md); see
[generic enums](../types/generics.md#generic-enums).

## See also

- [Types — enums](../types/enums.md)
- [Guards](guards.md)
- [Exhaustiveness](exhaustiveness.md)
