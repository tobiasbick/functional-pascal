# Documentation

This documentation specifies Functional Pascal for .NET. It is the design
baseline for the experimental compiler on this branch. Described behavior is
required of that compiler; implemented subsets are documented separately below.

| Area | Entry point |
| --- | --- |
| Design principles and scope | [Language overview](pascal/README.md) |
| Implemented compiler subset and first executable program | [First-program milestone](implementation/first-program.md) |
| Implemented arithmetic, conversions, branches, loops, and Mandelbrot | [Expressions and control flow](implementation/expressions-and-control-flow.md) |
| Multiple source files, unit imports, and visibility | [Multiple sources and units](implementation/multiple-sources.md) |
| CLR object/member access, nullable types, exceptions, and disposal | [CLR objects and resources](implementation/clr-objects-and-resources.md) |
| Arrays, args, closed CLR generic types, indexers, and enumeration | [Arrays and collections](implementation/arrays-and-collections.md) |
| Callable values, typed lambdas, closures, and generic CLR method calls | [Functions and closures](implementation/functions-and-closures.md) |
| Immutable records, defaults, methods, equality, and copy updates | [Records](implementation/records.md) |
| Nongeneric unions, exhaustive matching, patterns, and CLR type tests | [Unions and matching](implementation/unions-and-matching.md) |
| Source generic functions/record methods, constraints, and generic closures | [Generic routines](implementation/generic-routines.md) |
| Generic records/unions, recursive payloads, and CLR data interop | [Generic data types](implementation/generic-types.md) |
| Inferred routine/CLR calls and union constructors | [Type argument inference](implementation/type-argument-inference.md) |
| Synchronous contracts, managed references, optional/params parameters, and named calls | [Contracts and CLR parameters](implementation/contracts-and-parameters.md) |
| Source overloads, record CLR interfaces, portable symbols, and async functions/programs | [Overloads, interfaces, debugging, and async](implementation/overloads-interfaces-debugging-and-async.md) |
| Markdown source documentation, .NET XML sidecars, and SDK propagation | [XML API documentation](implementation/xml-documentation.md) |
| All primitive numbers, transparent aliases, enums, distinct values, and checked subranges | [Scalar types](implementation/scalar-types.md) |
| Global const/var fields, shared unit state, and CLR type initialization | [Global values](implementation/global-values.md) |
| Named lexical routines, recursion, captures, generics, and escaping delegates | [Nested routines](implementation/nested-routines.md) |
| Repeat/until and integral/subrange counting, including cleanup and async loops | [Typed loops](implementation/loops.md) |
| Implemented local SDK, `.fpasproj` builds, and mixed C#/FPAS projects | [MSBuild milestone](implementation/msbuild.md) |
| C# projects, xUnit tests, examples, and applications | [Repository layout plan](architecture/project-layout.md) |
| Lexical rules, bindings, and operators | [Basics](pascal/language/basics.md) |
| Source comments, Markdown, and .NET XML documentation | [Comments and API documentation](pascal/language/comments.md) |
| CLR types, records, unions, and collections | [Types](pascal/language/types.md) |
| Nominal scalars and checked ranges | [Data domains](pascal/language/data-domains.md) |
| Functions, delegates, and generics | [Functions](pascal/language/functions.md) |
| Preconditions and postconditions | [Contracts](pascal/language/contracts.md) |
| Branches, loops, and patterns | [Control flow](pascal/language/control-flow.md) |
| Exceptions, expected outcomes, and disposal | [Errors and resources](pascal/language/errors-and-resources.md) |
| Tasks and asynchronous execution | [Concurrency](pascal/language/concurrency.md) |
| Programs, units, imports, and global values | [Program structure](pascal/program-structure/README.md) |
| MSBuild projects and mixed-language solutions | [MSBuild integration plan](build/msbuild.md) |
| Calling and implementing .NET APIs | [.NET interoperability](pascal/interop/README.md) |
| Complete proposed programs | [Syntax examples](pascal/examples.md) |
| Direct CIL generation and runtime support | [CLR mapping](specs/clr-mapping.md) |
| Parser-facing syntax reference | [Syntax grammar](specs/syntax.md) |

Read the topic documents for semantic rules. The grammar defines surface syntax;
the mapping document defines emitted CLR identities. Examples illustrate those
rules and must not introduce additional language features.
