# Imports and companion sources

```pascal fragment
uses System as Sys,
  System.Console as Console,
  System.Collections.Generic as Collections,
  Game.Geometry as Geometry;
```

Each import has an explicit alias. Its target may be a CLR namespace, a CLR
non-generic type, or an FPAS unit container. Use `Console.WriteLine`,
`Collections.List<int>`, and `Geometry.Translate`. An import does not expose
unqualified members or acquire dependencies.

An alias is unique in its source and cannot be shadowed. Targets are resolved
case-sensitively against supplied assembly metadata and source declarations.
If a namespace and type identity make a target ambiguous, report an error rather
than choosing silently. Use a more precise target or a local type alias. Duplicate
imports of the same target are errors. Only primitive aliases are predefined;
library type names require an explicit import or a source declaration. Referencing
Fpas.Core for compiler-generated metadata does not import a source-level prelude.

Assemblies supply references; namespaces/types are import targets. These concepts
are not interchangeable. Duplicate qualified type identities across references
are an error in the baseline; extern assembly alias syntax is outside its scope.

### Two-file use

Save the [Geometry unit](units.md) as `Geometry.fpas` and this program as `Main.fpas`,
and supply both sources to one compilation:

```pascal project=geometry
program MovePoint;

uses Game.Geometry as Geometry, System.Console as Console;

begin
  const Start: Geometry.Point := new Geometry.Point(X := 1.0, Y := 2.0);
  const Moved: Geometry.Point := Geometry.Translate(Start, 3.0, 0.0);
  Console.WriteLine(Moved.X);
end;
end program;
```

The source filename does not declare the unit identity; the `unit` header does.
Two sources cannot declare the same qualified program/unit name.

## See also

- [Topic overview](README.md)
- [Handbook](../README.md)
- [Syntax grammar](../../specs/syntax.md)
