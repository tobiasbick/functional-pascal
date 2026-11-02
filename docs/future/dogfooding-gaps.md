# Dogfooding gaps

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
- [Records](../implementation/records.md) now support immutable data
  models, defaults, methods, and copy updates. [Unions and case matching](../implementation/unions-and-matching.md)
  now support domain outcomes and exhaustive inspection.
  [Generic records/unions](../implementation/generic-types.md) add reusable models,
  recursive payloads, and generic record methods with typed closures. Globals
  remain unavailable; use the supported subset for helpers.
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
