# Generic data types

Records, variants, and protocols can declare CLR type parameters, such as
`type Box of T` and `type Tree of T`. Use `of T` for one parameter or argument
and `of (K, V)` for multiple parameters or arguments. These source types are
invariant: `Box of string` and `Box of object` are distinct types. Recursive
references still need finite constructed values.

Single arguments can nest, as in `Box of List of int`. They include their own
dotted name and nullable annotation. To qualify or annotate the enclosing
type, write `(Box of int).Create(42)`, `(Outer of int).Inner`, or
`(Box of string)?`. `Box of string?` holds a nullable string.

Constraints follow the type declaration's equals sign and precede `record`, `variant`, or `protocol`.
They are checked at construction, in field/signature types, and when forwarding
parameters. A method may introduce its own type parameters without redeclaring an
enclosing parameter. Defaults must type-check for every valid instantiation.
Field and signature constraints use the complete source interface graph,
including forward/recursive record references and transparent aliases.
Constraints on all field shapes are checked before constructor eligibility, so a
forward-declared record with required fields cannot accidentally satisfy new().
Type parameters cannot shadow enclosing parameters, source types, imports,
predefined types, self, or args. Public constraints cannot expose private types.

Record construction keeps explicit owner arguments. Variant factories can infer
arguments from payloads and expected types; a fieldless generic case needs an
expected owner type. Value bindings still require their own annotations.
Matching substitutes the selected variant's arguments before checking payloads
and coverage. Public metadata preserves those CLR types across assembly boundaries.

## Example

```pascal
program TypeRules;

uses System as Sys;
uses System.Console as Console;

type Direction = enum: byte
  North = 0;
  South = 1;
end enum;

type Tree of T = variant
  Empty;
  Leaf(Value: T);
  Branch(Left: Tree of T; Right: Tree of T);
end variant;

function Compare of T(Left: T; Right: T): int;
where T: Sys.IComparable of T;
begin
  return Left.CompareTo(Right);
end function;

begin
  const Port: int? := null;
  Console.WriteLine(Port ?? 8080);

  const TreeValue: Tree of int := Tree.Branch(Tree.Leaf(1), Tree.Leaf(2));
  case TreeValue of
    when Tree.Empty:
      Console.WriteLine('Empty tree');
    when Tree.Leaf(const Value: int):
      Console.WriteLine(Value);
    when Tree.Branch(_, _):
      Console.WriteLine('Branch');
  end case;

  Console.WriteLine(Compare of int(2, 3));
  const Facing: Direction := cast 255 as Direction;
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

Nullable int, an application-defined variant, and a CLR enum are distinct types.
A cast can construct an unnamed enum value, which is why the final case requires
a fallback. Tree is an ordinary generic domain type, not an implicit library type.

## See also

- [Topic overview](README.md)
- [Handbook](../../README.md)
- [Syntax grammar](../../../specs/syntax.md)

## Source files

<details>
<summary>Browse 13 source files</summary>

- `src/Fpas.Compiler/Binding/Declarations/`
  - [SourceCatalog.TypeGenerics.cs](../../../../src/Fpas.Compiler/Binding/Declarations/SourceCatalog.TypeGenerics.cs)
- `src/Fpas.Compiler/Binding/References/`
  - [ReferenceCatalog.Constraints.cs](../../../../src/Fpas.Compiler/Binding/References/ReferenceCatalog.Constraints.cs)
  - [ReferenceCatalog.Generics.cs](../../../../src/Fpas.Compiler/Binding/References/ReferenceCatalog.Generics.cs)
  - [ReferenceCatalog.SourceGenerics.cs](../../../../src/Fpas.Compiler/Binding/References/ReferenceCatalog.SourceGenerics.cs)
- `src/Fpas.Compiler/Binding/Symbols/`
  - [GenericParameterSymbol.cs](../../../../src/Fpas.Compiler/Binding/Symbols/GenericParameterSymbol.cs)
- `src/Fpas.Compiler/Syntax/`
  - [Generics.cs](../../../../src/Fpas.Compiler/Syntax/Generics.cs)
- `src/Fpas.Compiler/Syntax/Parsing/`
  - [Parser.Generics.cs](../../../../src/Fpas.Compiler/Syntax/Parsing/Parser.Generics.cs)
- `tests/Fpas.Compiler.Tests/Types/`
  - [GenericDelimiterTests.cs](../../../../tests/Fpas.Compiler.Tests/Types/GenericDelimiterTests.cs)
  - [GenericTypeMetadataTests.cs](../../../../tests/Fpas.Compiler.Tests/Types/GenericTypeMetadataTests.cs)
  - [GenericTypeTests.cs](../../../../tests/Fpas.Compiler.Tests/Types/GenericTypeTests.cs)
- `tests/Fpas.IntegrationTests/Build/`
  - [MsBuildTests.GenericTypes.cs](../../../../tests/Fpas.IntegrationTests/Build/MsBuildTests.GenericTypes.cs)
  - [MsBuildTests.NestedTypes.cs](../../../../tests/Fpas.IntegrationTests/Build/MsBuildTests.NestedTypes.cs)
- `tests/Fpas.IntegrationTests/Execution/`
  - [GenericTypeExecutionTests.cs](../../../../tests/Fpas.IntegrationTests/Execution/GenericTypeExecutionTests.cs)

</details>
