# Development, API discovery, and runtime tools

Read this when building/testing the repository, finding an API, or investigating
compiler/program behavior. Start with the [development workflow](../../build/development.md)
for SDK requirements, all build/test commands, application bootstrap, and direct
CLI invocation. AGENTS.md contains repository policy; commands live in docs.

## Select the owning reference

| Task | Read |
| --- | --- |
| Build the C# compiler/tests, package the SDK, or select tests | [Development workflow](../../build/development.md) |
| Change project boundaries, fixtures, examples, or SDK packaging | [Repository layout](../../architecture/project-layout.md) |
| Bind/parser/type-check/lower a language feature | Its [handbook topic](../README.md#start-here), [grammar](../../specs/syntax.md), and [compiler architecture](../../architecture/README.md) |
| Change public types, signatures, dispatch, or emitted metadata | [Interop](../interop/README.md) and [CLR mapping](../../specs/clr-mapping.md) |
| Change project/reference/restore/incremental/output behavior | [MSBuild contract](../../build/msbuild.md) and [compiler output](../../architecture/compiler.md) |
| Decide which tests prove a behavior | [Test overview](../../../tests/README.md) and [acceptance map](../../../tests/coverage.md) |
| Distinguish implemented work, missing baseline work, and proposals | [Language scope](../getting-started/overview.md#initial-language-scope) and [language gaps](../../future/language-gaps.md) |
| Update this reference or add examples | [Maintenance](maintenance.md) |

Check the owning rule and its tests before claiming support; the grammar alone
is not proof that a proposed construct works. Public CLR names and
representations are governed by the owning specifications, updated in the same
change when the contract changes. The compiler emits CIL directly; runtime GC,
task infrastructure, collections, and library behavior belong to .NET.

## Discover APIs and compile sources

Use the selected package/framework version's .NET documentation, reference
metadata, and existing executable examples to establish signatures. Check CLR
names, generic arity, accessibility, parameter names/modes, overloads, and nullable
annotations. Use static extension-method calls. An unsupported signature must
produce a useful diagnostic rather than trigger reflection/dynamic invocation.

The CLI uses System.CommandLine; root help/version and compile help are available.
The compile command's help documents explicit references,
multiple sources, stdin, executable/library output, portable symbols, XML docs,
and JSON diagnostics. The [CLI contract](../program-structure/cli.md)
defines exit codes and publication behavior. Compile JSON reports are appropriate
for checking the diagnostic code and source location; test actual generated
programs when runtime behavior matters.

The C# compiler API also provides FpasCompiler.Analyze for semantic validation
of explicit source/reference inputs without emitted output, including async
liveness and optional documentation warnings. See the
[compiler API](../../architecture/compiler.md#compiler-api-and-output).

Project builds resolve framework, project, package, and Core dependencies through
MSBuild. Source uses statements never acquire them. Bootstrap the local SDK first
and handle its package-version cache as described by the development workflow.
Keep scratch programs and reports under .temp-data. Use the current CLR compiler
to validate these sources; another Pascal dialect cannot establish their behavior.

Use `fpas check <project.fpasproj|solution.slnx> --diagnostics json` for project
validation without analysis-owned assembly output. The selected SDK must match
the CLI. Preparation can restore/build dependencies; `--no-restore` skips only
restore. Project states and coverage distinguish invalid/blocked projects.
Input changes invalidate results. See the CLI reference for timeout and exit codes.
`fpas query definition|type|refs <context> --file <source> --line <n> --column <n>`
returns JSON from checked source positions (one-based UTF-16). Refs uses the whole
declared FPAS scope, with --limit and explicit truncation; linked files may require
--project. Prefer these queries for resolved identity/type facts over text search.

## Debug and profile

Use .NET tools supported in the actual environment. Portable PDBs carry source
documents, statement positions, lexical locals, closures, and async yield/resume
metadata. Preserve assembly/PDB pairs and source files when debugging generated
assemblies. The [debug information reference](../../architecture/debug-information.md)
describes supported symbols and stepping metadata.
The [debugging guide](../tools/debugging.md) describes IDE debugging, CLR value
views and external debugger prerequisites. `fpas debug --netcoredbg <absolute-path>`
hosts agent tools over stdio MCP using the official C# SDK and the separately
installed NetCoreDbg. Register it in the MCP host; a shell invocation alone does
not add tools. Launch an already built DLL, use current sessionId/stopId for
inspection and stepping, read target output through debug_output, and stop with
debug_stop. Follow the guide for tool order, output cursors and platform acceptance.

Investigate an actual program with a suitable .NET debugger/profiler before
writing external samplers or benchmark variants. Keep execution bounded and
assert outcomes; avoid depending on timer counts, network availability, or an
interactive terminal for ordinary regression tests. Isolate test files/processes.
Store performance output under .temp-data and distinguish measurements from
language guarantees. LSP, formatter and IDE project-system integration remain
separate work.

Write repository maintenance helpers in FPAS when the supported CLR APIs suffice.
If a concrete missing feature prevents this, document the practical limitation
in [dogfooding gaps](../../future/dogfooding-gaps.md). Finish the agreed baseline
before adding extensions; source syntax changes need the user's approval.
