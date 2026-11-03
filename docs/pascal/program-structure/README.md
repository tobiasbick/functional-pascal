# Programs, units, and imports

## Programs

```pascal
program Hello;

uses System.Console as Console;

function Greeting(Name: string): string;
begin
  return 'Hello, ' + Name;
end function;

begin
  Console.WriteLine(Greeting('CLR'));
end;
end program;
```

A program has a qualified name, optional imports, declarations, and one entry
body. An executable compilation has exactly one program; a library has none.
The entry body is a `begin ... end;` block. A separate `end program;` closes the
program declaration, matching the unit's named `end unit;` terminator. Both the
body's `end;` and the declaration's `end program;` are required for synchronous
and async programs. A terminal period cannot replace either terminator. Routines
retain their own named terminators.
`async program Name;` enables await in the entry body. Source selection and build
configuration are specified separately in the [MSBuild plan](../../build/msbuild.md).

The entry body receives an implicit immutable `args: string[]` binding, containing
command-line arguments excluding the executable name. Free functions do not
implicitly receive args. Entry completion yields process exit code zero;
unhandled exceptions follow the .NET host's failure behavior. The baseline has
no value-returning top-level entry body.

## Units

```pascal
unit Game.Geometry;

public type Point = record
  X: double;
  Y: double;
end record;

public function Translate(Value: Point; DX: double; DY: double): Point;
begin
  return Value with X := Value.X + DX; Y := Value.Y + DY; end with;
end function;

end unit;
```

A unit has no executable entry body. Top-level declarations are types, functions,
procedures, and initialized `const`/`var` bindings. There are no separate interface
sections or unit initialization/finalization blocks. Its required terminator is
`end unit;`.

Declarations are private unless marked `public`. Public signatures cannot expose
less-accessible types. Free functions are emitted as static methods on a unit
container. Nested language types remain ordinary CLR types. The emitted identities
are specified in [CLR mapping](../../specs/clr-mapping.md).

Signatures are available across the compilation without declaration-order lookup
restrictions. Value initializers execute once in source order in the unit's CLR
type initializer on first active use. Do not assume an application-wide eager
initialization order. Cyclic initializer dependencies retain CLR initialization
behavior; there is no source initialization scheduler.

### Global values

```pascal
unit Game.Statistics;

public const MaximumScore: int := 1_000;
var TotalMoves: long := 0L;

public procedure RecordMove();
begin
  TotalMoves := TotalMoves + 1L;
end procedure;

public function Moves(): long;
begin
  return TotalMoves;
end function;

end unit;
```

Each top-level binding requires an explicit type and initializer. Const emits a
static readonly field; var emits a mutable static field. Both default to unit-private
visibility and may be public. An imported public var is writable through its unit
alias and may be passed by ref/out under the ordinary addressability rules.

Initializers for both kinds run in written order in the same CLR type initializer.
There is no automatic locking, atomic increment, or task isolation. Shared global
state needs ordinary .NET synchronization. Contract conditions cannot read global
vars; use initialized parameters/captures or compile-time constants instead.
Source global const values are runtime readonly fields rather than CLR metadata
constants, even when initialized with literals. They do not qualify as constant
patterns, optional defaults, enum/subrange bounds, or contract operands.
Forward/cyclic initialization reads follow CLR default-storage semantics.

## Imports

```pascal
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

Save the Geometry unit above as `Geometry.fpas` and this program as `Main.fpas`,
and supply both sources to one compilation:

```pascal
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

## Build integration

Use the MSBuild `.fpasproj` format and `.slnx` solution integration from
the [build specification](../../build/msbuild.md). Compile items select sources;
ProjectReference and PackageReference supply dependencies. Source imports still
resolve names rather than acquiring libraries. The SDK supports ordinary restore,
build, clean/rebuild, and references between FPAS and C# projects.

IDE project loaders, LSP, editor/query integration, and publishing/deployment
profiles remain outside the initial build step.
