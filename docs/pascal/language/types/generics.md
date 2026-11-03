# Generic data types

Records and unions can declare CLR type parameters, such as Box<T> or Tree<T>.
Use angle brackets consistently in declarations, type annotations, and explicit
construction. These source types are invariant: Box<string> does not convert to
Box<object>. Recursive references still need finite constructed values.

Constraints follow the type declaration's equals sign and precede record/union.
They are checked at construction, in field/signature types, and when forwarding
parameters. A method may introduce its own type parameters without redeclaring an
enclosing parameter. Defaults must type-check for every valid instantiation.
Constraints on all field shapes are checked before constructor eligibility, so a
forward-declared record with required fields cannot accidentally satisfy new().
Type parameters cannot shadow enclosing parameters, source types, imports,
predefined types, self, or args. Public constraints cannot expose private types.

Record construction keeps explicit owner arguments. Union factories can infer
arguments from payloads and expected types; a fieldless generic case needs an
expected owner type. Value bindings still require their own annotations.
Matching substitutes the selected union's arguments before checking payloads
and coverage. Public metadata preserves those CLR types across assembly boundaries.

## Example

```pascal
program TypeRules;

uses System as Sys, System.Console as Console;

type Direction = enum : byte
  North = 0;
  South = 1;
end enum;

type Tree<T> = union
  Empty;
  Leaf(Value: T);
  Branch(Left: Tree<T>; Right: Tree<T>);
end union;

function Compare<T>(Left: T; Right: T): int;
where T: Sys.IComparable<T>;
begin
  return Left.CompareTo(Right);
end function;

begin
  const Port: int? := null;
  Console.WriteLine(Port ?? 8080);

  const TreeValue: Tree<int> := Tree.Branch(Tree.Leaf(1), Tree.Leaf(2));
  case TreeValue of
    when Tree.Empty:
      Console.WriteLine('Empty tree');
    when Tree.Leaf(const Value: int):
      Console.WriteLine(Value);
    when Tree.Branch(_, _):
      Console.WriteLine('Branch');
  end case;

  Console.WriteLine(Compare<int>(2, 3));
  const Facing: Direction := cast<Direction>(255);
  case Facing of
    when Direction.North:
      Console.WriteLine('North');
    when Direction.South:
      Console.WriteLine('South');
    else
      Console.WriteLine('An unnamed CLR enum value');
  end case;
end;
end program;
```

Nullable int, an application-defined union, and a CLR enum are distinct types.
A cast can construct an unnamed enum value, which is why the final case requires
a fallback. Tree is an ordinary generic domain type, not an implicit library type.

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)
