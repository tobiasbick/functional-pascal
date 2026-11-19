# Development, API discovery, and runtime tools

Start with the [development workflow](../../build/development.md) for SDK setup,
build/test commands and direct CLI invocation.
The [distribution reference](../../build/distribution.md) covers the standalone
Windows/Linux compiler, offline SDK export, executable SDK invocation and caches.
Use [application publishing](../../build/publishing.md) for `fpas publish`, Native
AOT prerequisites, delivery modes, output replacement and assembly/delivery signing.
Use [measurements](../tools/benchmarks.md) for benchmarks and
[diagnostic workflows](../tools/diagnostic-workflows.md) for traces, counters and dumps.
Use [diagnostic commands](../tools/cli-diagnostics.md), [terminal tests](../tools/terminal-testing.md)
and [JSON schemas](../tools/report-schemas.md) for the corresponding CLI contracts.

## Select the owning reference

| Task | Read |
| --- | --- |
| Change project boundaries, fixtures, examples, or SDK packaging | [Repository layout](../../architecture/project-layout.md) |
| Parse, bind, type-check or lower a language feature | Its [handbook topic](../README.md#start-here), [grammar](../../specs/syntax.md), and [compiler architecture](../../architecture/README.md) |
| Change public types, signatures, dispatch, or emitted metadata | [Interop](../interop/README.md) and [CLR mapping](../../specs/clr-mapping.md) |
| Change project/reference/restore/incremental/output behavior | [MSBuild contract](../../build/msbuild.md) and [compiler output](../../architecture/compiler.md) |
| Run or verify VS Code project, test and debug workflows | [VS Code extension guide](../../../src/VSCode/README.md#project-test-and-debug-workflows) |
| Preview and apply checked editor refactorings | [Refactoring reference](../tools/refactoring.md#vs-code-refactorings) |
| Decide which tests prove a behavior | [Test overview](../../../tests/README.md) and [acceptance map](../../../tests/coverage.md) |
| Build graphical or terminal applications | [GUI](../../ui/gui/README.md), [TUI](../../ui/tui/README.md), [MVU](../../ui/mvu/README.md) |
| Distinguish implemented work, missing baseline work, and proposals | [Language scope](../getting-started/overview.md#initial-language-scope) and [language gaps](../../future/language-gaps.md) |
| Update this reference or add examples | [Maintenance](maintenance.md) |

## Discover APIs and compile sources

Use `fpas env --json` to inspect compiler/SDK hashes, the selected .NET SDK and
debugger availability. `--directory` applies .NET SDK selection for another working
directory; cache inspection does not evaluate projects. See
[toolchain environment](../tools/environment.md) for limits and issue/exit codes.

Use `fpas doc <project|solution> --name System.Console.WriteLine --report json`
to find declarations and overloads. Pass the selected XML ID to
`fpas doc <project> --usage '<ID>' --report json` for a compiler-checked
unit, its required inputs, or the reason a template is unavailable. Use `M:`
for calls, `P:` for properties and indexers, and `F:` for fields. Member
access defaults to read. Add `--access write` for an assignment. Instance
members take the receiver as the first input. For repeated argument types,
usage templates prefer checked calls with parameter labels.
For generic APIs, repeat `--type-arg` for the enclosing types and then the
method, using FPAS primitives or fully qualified reference types.
An exact type name includes direct members. Narrow with --project/--assembly;
page with --limit/--offset and equal snapshotId. Metadata signatures are CLR
descriptions, not a guarantee of FPAS call support. Missing text is explicit.
See [API documentation lookup](../tools/api-documentation.md).

Use `fpas compile --help` for explicit references, inputs and outputs, or the
[CLI contract](../program-structure/cli.md) for exit codes. Check diagnostic codes
and locations with `--diagnostics json`. Execute the generated program to verify behavior.
Use structured diagnostic fields for expected/found values, related locations and
edits. Preview automatic fixes with `check --fix --dry-run`. `check --fix` applies
one validated batch and checks again. Remaining errors exit 1. See [diagnostics](../tools/diagnostics.md).

Use `refactor rename <context> --file <source> --line <n> --column <n> --new-name <name> --plan <path.json>`
to prepare a checked rename. Use `refactor remove-parameter <context> --file <source>
--line <n> --column <n> --plan <path.json>` for an unused private value parameter.
For parameter labels on an existing call, use `refactor name-arguments` with the same options.
Inspect the edits, then use `refactor apply <context>
--plan <path.json>`. --report json provides bounded reports. Binding changes and
stale plans are refused; source interface implementations are grouped together.
Only analyzed FPAS sources are edited; public API/foreign-consumer boundaries
remain explicit. See [checked refactorings](../tools/refactoring.md).

From C#, [FpasCompiler.Analyze](../../architecture/compiler.md#compiler-api-and-output)
checks explicit sources and references in memory. Project builds resolve
dependencies through MSBuild. Keep scratch programs and reports under `.temp-data`.

Use the [project commands](../program-structure/project-commands.md) for init,
restore, build, run and clean. Preview project, library, test or solution templates
with `--dry-run --report json`. `fpas target <context>` reports the executable path
without a restore or build. Use `--project` for an ambiguous solution.
`fpas run` forwards arguments after `--` and preserves the program's exit code.
JSON mode captures output and inherits stdin. Set `--timeout` for a bounded run.
Use `run --failure-report failure.json` to save the first unhandled exception
with the shared debugger report, then resume the same process to its natural exit.
The destination must be new; matching PDBs/sources and the supported CoreCLR
debugger platform are required. Available locals and async stack context are
bounded; global task enumeration is unavailable. See the run reference for limits.

Use `fpas init test Tests --framework xunit` to create a native test library;
mstest/nunit are also supported. `--framework mstest --runner mtp` creates a
self-hosted FPAS test application. Use existing framework attributes and assertions.
`fpas test <project|solution> --report json` builds and runs through VSTest or MTP.
Runs produce TRX. Use `--list` for discovery. MTP can return a test UID when method
metadata is absent. MTP filters/settings require the runner's extension.
`--no-build` uses existing outputs. The default deadline is 120s.
See [testing](../tools/testing.md) for frameworks, result fields and limits.

Use `fpas fmt <paths>` to format sources, directories, projects or solutions.
`--check --list` lists changed paths and `--stdout` previews one source.
See [formatting](../tools/fmt-style.md) for reports and exit codes.

Use `fpas check <project.fpasproj|solution.slnx> --diagnostics json` for project
validation without analysis-owned assembly output. The selected SDK must match
the CLI. Preparation can restore/build dependencies; `--no-restore` skips only
restore. Project states and coverage distinguish invalid/blocked projects.
Input changes invalidate results. See the CLI reference for timeout and exit codes.
`fpas query definition|type|refs|inspect <context> --file <source> --line <n> --column <n>`
returns JSON from checked source positions (one-based UTF-16). Refs uses the whole
declared FPAS scope, with --limit and explicit truncation; linked files may require
--project. Prefer these queries for resolved identity/type facts over text search.
Queries can retain checked facts when other regions fail. Inspect
coverage.complete and coverage.semantics before treating searches as exhaustive;
analysis_unavailable distinguishes unverified regions from not_found. Rename
requires successful complete analysis. See the [CLI contract](../program-structure/cli.md).
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

`fpas lsp` exposes stdio diagnostics, navigation, IntelliSense and checked edits.
Use zero-based UTF-16 positions in LSP and one-based positions in CLI queries.
Select the project or solution by absolute path or local file URI in
`initializationOptions.context`. Restore and
foreign-project builds remain explicit. See the
[language-server reference](../tools/language-server.md) for lifecycle and limits.
`fpas env --json` reports its packaged identity without starting a session.

## Debug and profile

Keep the original sources, assembly and matching portable PDB together.
The [debug information reference](../../architecture/debug-information.md) covers
stepping, locals, closures and async metadata.
The [debugging guide](../tools/debugging.md) describes IDE debugging, CLR value
views and packaged debugger requirements. Use `fpas debug Program.dll --cwd src --break
main.fpas:12 --print Value` when the source is in `src`. Breakpoint paths resolve
against `--cwd`, which defaults to the DLL directory. Batch capture stops at the
first requested breakpoint or unhandled exception and terminates its target.
Use `--tracepoint` for repeated observations. Inspection is passive.
Target stdin defaults to EOF. See the
[batch contract](../tools/batch-debugging.md) for limits, paths and exit codes.
Editors use `fpas debug --protocol dap` with the same core and passive evaluator.

Profile the actual program with supported .NET tools. Keep measurements under
`.temp-data` and bound test runs.
