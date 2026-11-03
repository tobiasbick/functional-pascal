# Functional Pascal for .NET

A function-first Pascal language with selected Ada conventions, compiling
directly to CLR assemblies, portable debug information, and .NET metadata.
Functions, immutable records, and discriminated unions are its primary
abstractions. The CLR provides managed memory and task infrastructure.

This handbook describes the language and its supported .NET workflows. Known
baseline omissions and deferred extensions are tracked in
[language gaps](../future/language-gaps.md). The
[architecture reference](../architecture/README.md) explains the compiler;
[test coverage](../../tests/coverage.md) identifies verified behavior.

## Design principles

- Explicit binding mutability: const prevents rebinding; var permits assignment.
- Explicit types on every binding, parameter, result, field, and pattern binding.
- Case-sensitive names and lowercase keywords, preserving actual CLR identities.
- Pascal blocks and named terminators, with ordinary .NET APIs and reference behavior.
- Executable contracts, exhaustive union matching, and explicit resource cleanup.

## Documentation areas

| Area | Hub | Topics |
| --- | --- | --- |
| Agent reference | [agents](agents/README.md) | Compact authoring rules and verified examples |
| Getting started | [getting-started](getting-started/README.md) | Overview, hello world, first program, keywords |
| Language | [language](language/README.md) | Basics, types, routines, control flow, patterns, errors, async |
| Program structure | [program-structure](program-structure/README.md) | Programs, units, imports, projects, compiled output, CLI |
| .NET interoperability | [interop](interop/README.md) | CLR members, delegates, interfaces, events, attributes, limits |
| .NET libraries | [libraries](libraries/README.md) | Collections, text/regex, files, JSON, and library selection |
| Tools | [tools](tools/README.md) | Diagnostics, documentation checks, debugging, and profiling |
| Examples | [examples](examples/README.md) | Complete programs grouped by task |
| Formal syntax | [syntax grammar](../specs/syntax.md) | Parser-facing syntax and semantic boundaries |
| CLR representation | [CLR mapping](../specs/clr-mapping.md) | Emitted identities, signatures, and runtime boundary |

## Start here

Agents can start with the [compact reference](agents/README.md) and load only the
topics relevant to their task. For a progressive introduction:

1. [Getting started](getting-started/README.md): language goals and a runnable project.
2. [Basics](language/basics/README.md): values, bindings, operators, and conversions.
3. [Control flow](language/control-flow/README.md): branches and typed loops.
4. [Functions](language/functions/README.md): calls, delegates, generics, and contracts.
5. [Types](language/types/README.md): records, unions, domains, arrays, and nullability.
6. [Pattern matching](language/pattern-matching/README.md): payloads, guards, and coverage.
7. [Errors and resources](language/error-handling/README.md): exceptions and disposal.
8. [Concurrency](language/concurrency/README.md): tasks, await, and cancellation.
9. [Program structure](program-structure/README.md): units, imports, and project builds.
10. [Interop](interop/README.md) and [libraries](libraries/README.md): consume .NET APIs.
11. [Tools](tools/README.md): validate, debug, and inspect actual programs.

Repository restore/build/test commands belong in the
[development workflow](../build/development.md). Complete Pascal programs and
units in this handbook are checked directly by the documentation tests;
[example conventions](tools/documentation-examples.md) distinguish complete
sources from explicitly marked syntax fragments.
