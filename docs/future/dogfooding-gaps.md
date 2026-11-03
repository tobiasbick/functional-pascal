# Dogfooding gaps

For the completion order of the agreed language baseline and the deferred
source-interface proposal, see [language gaps](language-gaps.md). This document
records practical limits when using the implemented compiler for maintenance.

The [compiler subset](../implementation/first-program.md) and
[MSBuild milestone](../implementation/msbuild.md) execute small FPAS programs
and libraries. These concrete limits currently require host tools:

- Directory traversal, arrays/indexers, generic collections, args, callable
  values, closures, and explicit generic .NET method calls now work. Write
  filesystem/text maintenance helpers in FPAS. The
  [directory inventory](../../examples/directory-inventory/README.md) uses real
  LINQ Where/Select and a mutable captured total. Extension methods are called
  in static form. [Source generic functions](../implementation/generic-routines.md)
  now support typed Map/Compose helpers and shared reusable routines.
  [Call/union inference](../implementation/type-argument-inference.md) now closes
  reusable routines and union constructors from typed inputs and context.
  [Managed parameter modes and synchronous contracts](../implementation/contracts-and-parameters.md)
  support direct CLR calls and local validation. [Async functions and programs](../implementation/overloads-interfaces-debugging-and-async.md)
  now await Task/ValueTask APIs and asynchronous disposal directly. Source
  overloads and record implementations of CLR interfaces are also available.
- [Imported CLR events](../implementation/events.md) now support typed handlers,
  subscription/removal, and ordinary .NET accessor calls, including async code.
  Retain the delegate for later removal and use explicit cleanup when needed.
- [Runtime type expressions](../implementation/typeof.md) now provide typeof(T)
  for supported concrete/generic types, including inside closures and async code.
  Helpers can pass actual Type objects to .NET APIs without constructing values.
- [Source attributes](../implementation/attributes.md) now annotate real .NET
  APIs and data models, including record property names consumed by serializers.
  Values must be legal metadata constants, closed typeof values, or array literals.
- [Records](../implementation/records.md) now support immutable data
  models, defaults, methods, and copy updates. [Unions and case matching](../implementation/unions-and-matching.md)
  now support domain outcomes and exhaustive inspection.
  [Generic records/unions](../implementation/generic-types.md) add reusable models,
  recursive payloads, and generic record methods with typed closures.
- [Global values](../implementation/global-values.md) now support shared unit
  state through ordinary static readonly/mutable fields and CLR initialization.
  Helpers can share configuration and counters without replacing field reads
  with copied closure captures.
- [Named nested routines](../implementation/nested-routines.md) now support lexical
  helpers, recursive algorithms, overloads/generics, managed parameters, contracts,
  and async calls. Initialize captured bindings before calls/delegate conversion.
- [Typed loops](../implementation/loops.md) now support repeat/until and all
  primitive integral/subrange counters, including native widths, extreme
  endpoints, per-iteration captures, cleanup, and asynchronous suspension.
- [Primitive numbers and scalar domains](../implementation/scalar-types.md) now
  support decimal/unsigned/native values, transparent aliases, actual CLR enums,
  distinct identities, and checked integral ranges. Numeric maintenance helpers
  can use these directly, including arrays and generic collections.
- Portable PDB generation now preserves statement locations, local scopes,
  closure methods, and async yield/resume metadata. Use supported .NET debuggers;
  automatic IDE/project integration remains pending.
- [XML API documentation](../implementation/xml-documentation.md) now exports
  attached Markdown comments through ordinary compiler/SDK options. Use that
  output for supported public APIs; source hover/completion remains separate work.

Multi-source programs and libraries now work, including public/private functions
and unit imports. The SDK supplies framework/project/package references through
normal .NET resolution; direct CLI invocation still needs explicit references.

These gaps do not justify validating new FPAS with the former VM/compiler or
claiming that the complete language/build integration already works.
