# Discriminated unions

A union defines a closed set of named cases with optional immutable payloads.

```pascal fragment context=declarations id=shape
public type Shape = union
  Dot;
  Circle(Radius: double);
  Rectangle(Width: double; Height: double);
end union;
```

Construct values with `Shape.Dot`, `Shape.Circle(2.0)`, or
`Shape.Rectangle(Width := 3.0, Height := 4.0)`. A fieldless case has no parentheses.
Payload fields are immutable. Case names and payload labels are case-sensitive.

A union is a closed nominal reference type with compiler-generated case
representations. It is **not** `System.Enum`. Generic unions use ordinary type
parameters. `Tree<T>` may recursively refer to `Tree<T>` because CLR references
break the storage cycle. All paths to constructing finite values must still
follow the declared constructors; null is not a union case.

Generic unions are declared by applications or imported from libraries like
other types. For example:

```pascal fragment context=declarations
type Tree<T> = union
  Empty;
  Leaf(Value: T);
  Branch(Left: Tree<T>; Right: Tree<T>);
end union;
```

Constructors select the uninstantiated union owner, for example `Tree.Leaf(1)`.
Type arguments are inferred from payloads and expected type. A fieldless generic
case requires expected type, such as `const Empty: Tree<int> := Tree.Empty;`.
Constructor inference must resolve exactly one union type; arbitrary class static
methods do not gain this shorthand. Value declarations still require explicit
types even when a constructor supplies enough information to infer its arguments.

Union equality compares the case and its fields using the record field rules.
[Control flow](../control-flow/README.md) specifies exhaustive matching.

## Example

```pascal
program MoveOutcomes;

uses System.Console as Console;

type Position = record
  public X: int;
  public Y: int;
end record;

type MoveOutcome = union
  Moved(NewPosition: Position);
  Blocked(Reason: string);
end union;

function Move(Current: Position; DX: int; DY: int): MoveOutcome;
begin
  const Next: Position := Current with
    X := Current.X + DX;
    Y := Current.Y + DY;
  end with;
  if Next.X < 0 or Next.X >= 10 or Next.Y < 0 or Next.Y >= 10 then
    return MoveOutcome.Blocked('The edge of the map');
  end if;
  return MoveOutcome.Moved(Next);
end function;

begin
  const Start: Position := new Position(X := 0, Y := 0);
  case Move(Start, -1, 0) of
    when MoveOutcome.Moved(const Next: Position):
      Console.WriteLine(Next.X);
    when MoveOutcome.Blocked(const Reason: string):
      Console.WriteLine(Reason); // The edge of the map
  end case;
end;
end program;
```

Both cases are normal gameplay outcomes. The union is declared by this application
and handled explicitly. It does not throw, unwind, or return early on its own.
Arithmetic overflow still throws an ordinary CLR OverflowException.

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)
