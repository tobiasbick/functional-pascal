# Functional Pascal for .NET

A function-first language with Pascal-style syntax, targeting the Common Language
Runtime directly through CIL and assembly metadata.

This branch contains a **language specification and a direct C# compiler**. The current
subset compiles programs and units with typed bindings, numeric/Boolean expressions,
named routines, control flow, CLR objects, nullable values, exceptions, arrays, and generic .NET collections directly to CIL. Remaining baseline work is tracked
in [language gaps](docs/future/language-gaps.md). See the [compiler subset](docs/implementation/first-program.md)
and [implemented MSBuild integration](docs/implementation/msbuild.md) for the
exact supported behavior.

[CLR comparisons and acceptance](docs/implementation/comparisons-and-acceptance.md)
cover imported comparison operators, precise unsupported-signature diagnostics,
executable handbook examples, and the verified net10.0/AnyCPU build contract.

[Scalar types](docs/implementation/scalar-types.md) include all specified CLR
primitive numbers, transparent aliases, real CLR enums, distinct values, and
checked integral subranges. The [data-domains example](examples/data-domains/README.md)
demonstrates nominal identities, decimal values, and valid subrange defaults.

[Global values](docs/implementation/global-values.md) add readonly/mutable static
fields with CLR type initialization. The
[shared-state example](examples/shared-state/README.md) shares unit state through
functions, delegates, and ref arguments.

[Named nested routines](docs/implementation/nested-routines.md) support lexical
helpers, recursion, captures, overloads, generics, contracts, and async methods.
The [local-functions example](examples/local-functions/README.md) returns a local
counter as an ordinary CLR delegate.

[Typed loops](docs/implementation/loops.md) support repeat/until and counting
with all integral types and subranges, including guarded extreme endpoints.
The [loops example](examples/loops/README.md) also runs these across await.

[CLR events](docs/implementation/events.md) support subscribe/unsubscribe with
ordinary .NET delegates and sync/async cleanup. The
[events example](examples/events/README.md) listens to a real collection change.

[Runtime type expressions](docs/implementation/typeof.md) provide typeof(T)
without constructing instances. The
[type-inspection example](examples/type-inspection/README.md) uses CLR type identity.

[Source attributes](docs/implementation/attributes.md) emit real CLR metadata on
types, routines, and record properties. The
[attributes example](examples/attributes/README.md) changes JSON property names.

The [Mandelbrot example](examples/mandelbrot/README.md) is executable through its
`.fpasproj`; its raster output is verified by the integration tests. The
[expression/control-flow milestone](docs/implementation/expressions-and-control-flow.md)
describes arithmetic, casts, scopes, branches, and loops now available.

Projects also support [multiple sources and units](docs/implementation/multiple-sources.md).
The [Units example](examples/units/README.md) compiles a program and two units into
one assembly, using public functions and private helpers with explicit imports.

[CLR objects and resources](docs/implementation/clr-objects-and-resources.md) adds
constructors, instance members, nullable flow, and try/catch/finally/using const.
The [file-reading example](examples/read-file/README.md) uses StreamReader through
the actual .NET APIs and builds through its `.fpasproj`.

[Arrays and collections](docs/implementation/arrays-and-collections.md) adds args,
UTF-16 indexing, closed generic .NET types, indexers, and for-in enumeration with
disposal. The [directory inventory](examples/directory-inventory/README.md) takes
CLI arguments, collects file paths in List<string>, and processes text files.

[Functions and closures](docs/implementation/functions-and-closures.md) adds
callable values, typed lambdas, shared mutable captures, CLR method groups, and
explicit generic .NET method calls. The inventory uses actual LINQ Where/Select.

[Records](docs/implementation/records.md) adds immutable source types, defaults,
private state, instance/static methods, value equality, and with updates. The
[inventory model](examples/inventory-model/README.md) demonstrates items and positions.

[Unions and matching](docs/implementation/unions-and-matching.md) adds typed cases,
exhaustive case statements/expressions, nested/guarded patterns, and CLR type tests.
The [game-actions example](examples/game-actions/README.md) models movement and item outcomes.

[Generic routines](docs/implementation/generic-routines.md) adds source type parameters,
explicit generic calls, where constraints, and typed generic closures. The
[generic pipeline](examples/generic-pipeline/README.md) demonstrates Map and Compose.

[Generic data types](docs/implementation/generic-types.md) add records and recursive
unions with CLR type parameters, constraints, defaults, and cross-assembly matching.
The [generic tree](examples/generic-tree/README.md) maps and renders a typed tree.
[Type argument inference](docs/implementation/type-argument-inference.md) now closes
routine/CLR calls from their arguments and union constructors from payloads and
expected types, while declarations retain mandatory type annotations.

[Contracts and CLR parameters](docs/implementation/contracts-and-parameters.md)
add synchronous preconditions/postconditions, managed `ref`/`out`/`in` arguments,
optional and `params` parameters, and named routine/CLR calls. The emitted checks
and parameter metadata also apply to C# consumers and delegates.

[Overloads, interfaces, debugging, and async](docs/implementation/overloads-interfaces-debugging-and-async.md)
add source overload groups, records implementing CLR interfaces, portable PDBs,
and Task-based async functions/lambdas/programs with await using and contracts.

[XML API documentation](docs/implementation/xml-documentation.md) exports attached
Markdown comments with CLR documentation IDs, warning policy, and normal SDK
output/reference copying.

Start with the [documentation hub](docs/README.md), the
[language overview](docs/pascal/README.md), and the
[complete syntax examples](docs/pascal/examples.md).
Agents can use the compact [task-oriented reference](docs/pascal/agents/README.md);
repository commands are collected in the [development workflow](docs/build/development.md).

The locally packaged `Fpas.NET.Sdk` builds `.fpasproj` projects in mixed `.slnx`
solutions. C# projects can consume FPAS libraries, and FPAS can consume resolved
project/package references. The [MSBuild plan](docs/build/msbuild.md) describes
the wider integration goal; IDE integration remains later work.

The [repository layout](docs/architecture/project-layout.md) specifies one
Fpas.slnx solution, C# projects under src, xUnit tests under tests, SDK files under
sdk, and examples/apps directories. The Hello example is executable; apps remain
reserved for later work.

## Build and test

Use a stable .NET 10 SDK. global.json selects 10.0.100 or a later installed 10.0
feature band. Package versions are centralized in Directory.Packages.props.
The [development workflow](docs/build/development.md) contains restore/build/test
commands, focused documentation checks, application SDK bootstrap, and CLI use.
The [test overview](tests/README.md) explains compiler, execution, metadata, and
mixed-project coverage. The SDK is resolved from the local feed; follow the
documented version/cache workflow when changing its packaged tools.
