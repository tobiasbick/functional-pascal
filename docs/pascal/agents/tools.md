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

Use `fpas env --json` to inspect compiler/SDK hashes, the selected .NET SDK and
debugger availability. `--directory` applies .NET SDK selection for another working
directory; cache inspection does not evaluate projects. See
[toolchain environment](../tools/environment.md) for limits and issue/exit codes.

Use the selected package/framework version's .NET documentation, reference
metadata, and existing executable examples to establish signatures. Check CLR
names, generic arity, accessibility, parameter names/modes, overloads, and nullable
annotations. Use static extension-method calls. An unsupported signature must
produce a useful diagnostic rather than trigger reflection/dynamic invocation.

Use `fpas doc <project|solution> --name System.Console.WriteLine --report json`
to find actual FPAS/.NET declarations, overloads and source Markdown/reference XML.
An exact type name includes direct members. Narrow with --project/--assembly;
page with --limit/--offset and equal snapshotId. Metadata signatures are CLR
descriptions, not a guarantee of FPAS call support. Missing text is explicit.
See [API documentation lookup](../tools/api-documentation.md).

The CLI uses System.CommandLine; root help/version and compile help are available.
The compile command's help documents explicit references,
multiple sources, stdin, executable/library output, portable symbols, XML docs,
and JSON diagnostics. The [CLI contract](../program-structure/cli.md)
defines exit codes and publication behavior. Compile JSON reports are appropriate
for checking the diagnostic code and source location; test actual generated
programs when runtime behavior matters.
Diagnostics also supply expected/found data, help, related notes and exact proposed
edits where available. Use these fields rather than parsing messages.
Use `check --fix --dry-run` to preview automatic corrections; `check --fix`
applies one batch with original-text/context validation and a fresh final check.
Review proposals stay untouched. Remaining errors still exit 1; JSON fixes/file
flags distinguish applied changes from check success. Non-source failures have
no location. See [diagnostics](../tools/diagnostics.md).

Use `refactor rename <context> --file <source> --line <n> --column <n> --new-name <name> --plan <path.json>`
to prepare a checked rename. Inspect the plan, then use `refactor apply <context>
--plan <path.json>`. --report json provides bounded reports. Binding changes and
stale plans are refused; source interface implementations are grouped together.
Only analyzed FPAS sources are edited; public API/foreign-consumer boundaries
remain explicit. See [renaming](../tools/refactoring.md).

The C# compiler API also provides FpasCompiler.Analyze for semantic validation
of explicit source/reference inputs without emitted output, including async
liveness and optional documentation warnings. See the
[compiler API](../../architecture/compiler.md#compiler-api-and-output).

Project builds resolve framework, project, package, and Core dependencies through
MSBuild. Source uses statements never acquire them. Bootstrap the local SDK first
and handle its package-version cache as described by the development workflow.
Keep scratch programs and reports under .temp-data. Use the current CLR compiler
to validate these sources; another Pascal dialect cannot establish their behavior.

Use [init/build/run](../program-structure/project-commands.md) for project workflows.
Init has project/library/solution templates, --dry-run and an exact JSON file plan;
--sdk-feed/--sdk-version configure project templates. Build invokes MSBuild;
run selects an executable (use --project when a solution is ambiguous), forwards
arguments after -- and preserves its exit code. --report json captures program
output separately; stdin remains inherited. Run defaults to no deadline; select
--timeout for bounded automation. No command overwrites conflicting init files.

Use `fpas init test Tests --framework xunit` to create a native test library;
mstest/nunit are also supported. Use existing framework attributes and assertions.
`fpas test <project|solution> --report json` builds and runs through VSTest/TRX;
--list discovers method identities, --filter/--settings select native behavior,
--no-build uses existing outputs. The default whole-operation deadline is 120s.
See [testing](../tools/testing.md) for frameworks, result fields and limits.

Use `fpas fmt --check <paths> --report json` to inspect fixed-style formatting,
then `fpas fmt <paths>` to apply it. Sources, directories, evaluated projects and
solutions are supported. --check --list prints changed paths; --stdout prints
one formatted source without writes. Formatting preserves tokens/comments and
requires complete syntax. See [formatting](../tools/fmt-style.md) for scope,
publication and exit codes.

Use `fpas check <project.fpasproj|solution.slnx> --diagnostics json` for project
validation without analysis-owned assembly output. The selected SDK must match
the CLI. Preparation can restore/build dependencies; `--no-restore` skips only
restore. Project states and coverage distinguish invalid/blocked projects.
Input changes invalidate results. See the CLI reference for timeout and exit codes.
`fpas query definition|type|refs|inspect <context> --file <source> --line <n> --column <n>`
returns JSON from checked source positions (one-based UTF-16). Refs uses the whole
declared FPAS scope, with --limit and explicit truncation; linked files may require
--project. Prefer these queries for resolved identity/type facts over text search.
`query find <context> --name <text>` searches explicit checked FPAS declarations;
use --match exact|prefix|contains, repeat --kind, and page with --limit/--offset.
Find's --project narrows searched owners. Combine pages only with equal snapshotId.
Use returned navigation spans with inspect for exact declaration text, attached
documentation and direct type context. Inspect's --limit bounds related source
declarations; copied text shares a 64 KiB budget with explicit omissions. Neither
query searches C#/F# source or guesses source for external metadata.
`query callees|callers <context> --file <source> --line <n> --column <n>`
returns outgoing body operations or incoming sites across the checked FPAS scope.
Default selection follows the exact callable target; `--body` selects the
innermost routine/lambda/entry/initializer region. Callers accepts routine/lambda
bodies only. Use --limit/--offset and equal snapshotId when paging. Sites separate
invocations from callable references and retain static dispatch uncertainty;
unknown delegate calls are not attributed to guessed functions. Metadata-only
and bodyless outgoing targets omit site counts. See the
[call-query contract](../program-structure/cli.md#callers-and-callees) for scope,
exact constructor/accessor anchors and excluded runtime protocols.

## Debug and profile

Use .NET tools supported in the actual environment. Portable PDBs carry source
documents, statement positions, lexical locals, closures, and async yield/resume
metadata. Preserve assembly/PDB pairs and source files when debugging generated
assemblies. The [debug information reference](../../architecture/debug-information.md)
describes supported symbols and stepping metadata.
The [debugging guide](../tools/debugging.md) describes IDE debugging, CLR value
views and packaged debugger requirements. Use `fpas debug Program.dll --break
main.fpas:12 --print Value` for one bounded detached JSON report. The command
captures the first requested stop and terminates its owned target. It never
builds/restores or executes getters/methods during inspection. `--stdin file`
supplies explicit target input; the default is EOF. See the
[batch contract](../tools/batch-debugging.md) for limits, paths and exit codes.
Editors use `fpas debug --protocol dap` with the same core and passive evaluator.

Investigate an actual program with a suitable .NET debugger/profiler before
writing external samplers or benchmark variants. Keep execution bounded and
assert outcomes; avoid depending on timer counts, network availability, or an
interactive terminal for ordinary regression tests. Isolate test files/processes.
Store performance output under .temp-data and distinguish measurements from
language guarantees. LSP and IDE project-system integration remain separate work.

Write repository maintenance helpers in FPAS when the supported CLR APIs suffice.
If a concrete missing feature prevents this, document the practical limitation
in [dogfooding gaps](../../future/dogfooding-gaps.md). Finish the agreed baseline
before adding extensions; source syntax changes need the user's approval.
