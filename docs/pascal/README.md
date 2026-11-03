# Functional Pascal for .NET

Functional Pascal is a function-first language using Pascal's statement and block
structure. It compiles directly to Common Intermediate Language (CIL), CLR
metadata, and portable debug information. It does not generate C# as an
intermediate source language.

**Status: implemented language features with known baseline gaps.** The
[compiler overview](../implementation/first-program.md) and topic milestones
describe support; [language gaps](../future/language-gaps.md) lists remaining
baseline work and deferred proposals. Complete handbook examples are compiled
and executed by the [acceptance tests](../implementation/comparisons-and-acceptance.md).
The [MSBuild milestone](../implementation/msbuild.md) describes project builds.

## Design decisions

- Free functions, procedures, immutable records, and discriminated unions are the
  primary abstractions. Effects are allowed; functions are not implicitly pure.
- Distinct scalars and checked integral subranges express named data domains.
  Records support defaults, private members, and static factories/methods.
  Explicitly typed globals may be const or var.
- Keep `begin`, `end function`, `end if`, `:=`, `uses`, and `case ... when`.
  Program entry blocks close with `end;`, followed by `end program;` for the
  program declaration. Units close with `end unit;`.
- Use C# primitive aliases and ordinary CLR generic types: `int`, `double`,
  `List<int>`, `Task<string>`. `integer` and `real` are not predefined aliases.
- The language is case-sensitive, including keywords. This preserves CLR names
  without case-folding collisions. Keywords use lowercase spelling.
- Source comments use `//`. Attached Markdown documents declarations, with
  optional .NET XML documentation output for referenced APIs.
- `const` prevents rebinding; `var` permits it. Neither makes a referenced .NET
  object deeply immutable.
- Every user-declared value binding has an explicit type, including local
  values, loop variables, resources, and pattern bindings. Initializers do not
  substitute for type annotations.
- Arrays and imported collections have their CLR reference behavior. No implicit
  copy-on-write collection layer is inserted.
- Asynchronous code uses `async`, `await`, .NET tasks, and cooperative cancellation.
  There is no `go`, language scheduler, or implicit task-ownership scope.
- Consume CLR classes, structs, interfaces, properties, delegates, and events
  directly from referenced assemblies; namespaces are imported explicitly.
- Preserve precise static typing, checked integral arithmetic by default, and
  exhaustive matching of language-defined unions.
- `requires` and `ensures` express preconditions and postconditions, checked in
  every build and enforced through ordinary CLR exceptions.
- Use CLR exceptions for failures and CLR nullability for absent values.
  Application-specific unions describe domain outcomes. There is no implicit
  Option/Result prelude or failure-propagation operator.
- Declare resources with `using const` or `await using const`, without a separate
  block closer; cleanup runs at the end of the enclosing lexical scope.
- Use a small `Fpas.Core` for language-specific support. Do not mirror the .NET
  class library in a separate `Std.*` API catalog.

## Source shape

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

Public declarations belong in units. Values, parameters, function results, and
record fields have explicit types. Generic argument inference does not permit
omitting a value binding's type. See
[program structure](program-structure/README.md).

## Initial language scope

The baseline includes functions and closures, contracts, CLR generics, records,
distinct types, subranges, unions, global values, exceptions, disposal, delegates,
events, method-based interface implementation, and asynchronous methods.
It supports ordinary managed .NET APIs.

It does not initially include user-authored class hierarchies, user-authored
interface declarations, unsafe pointers, `ref struct` values, spans, ref returns,
byref locals, expression-tree conversion, custom awaiter patterns, async iterators,
dynamic binding, operator declarations, or compiler plug-ins. An unsupported
metadata signature produces a diagnostic identifying the unsupported feature.
This scope does not promise access to every possible .NET API.

Finish this baseline before adding language extensions. The
[language gaps](../future/language-gaps.md) track confirmed missing baseline work
and the separate, deferred proposal for source interface declarations.

The [MSBuild specification](../build/msbuild.md) defines `.fpasproj` projects,
`.slnx` solutions, standard dependency restore, and cross-language project
references as the initial build scope. The local SDK supports and tests this
contract for net10.0 and AnyCPU; see the
[acceptance coverage](../implementation/comparisons-and-acceptance.md#build-acceptance).
Publishing and deployment profiles remain later work.

The [repository layout](../architecture/project-layout.md) defines the C#
compiler/toolchain projects, xUnit tests, and examples/apps directories.

Visual Studio project-system integration, LSP, VS Code integration, query tools,
formatter integration, and debugger tooling remain outside this step. Their
omission is not a decision to remove those tools.

## Reading order

1. [Basics](language/basics.md)
   and [comments/API documentation](language/comments.md)
2. [Types](language/types.md)
   and [data domains](language/data-domains.md)
3. [Functions](language/functions.md)
4. [Contracts](language/contracts.md)
5. [Control flow](language/control-flow.md)
6. [Errors and resources](language/errors-and-resources.md)
7. [Concurrency](language/concurrency.md)
8. [.NET interoperability](interop/README.md)
9. [Complete examples](examples.md)

The [grammar](../specs/syntax.md) and [CLR mapping](../specs/clr-mapping.md)
are implementation references. Syntax choices in this baseline are decisions;
features outside its scope require a separate specification.
