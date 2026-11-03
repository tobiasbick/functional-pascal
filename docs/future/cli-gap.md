# CLR CLI gaps

This document records CLI capabilities missing from the CLR port and possible
CLR equivalents for the Rust toolchain. It is an inventory, not an implementation
plan. It does not approve implementation, define an order of work, select
dependencies, or establish new command syntax or language semantics.

Command spellings below describe the Rust CLI or illustrate possible CLR
commands. They are not instructions for the current CLR executable. Capability
parity does not require preserving VM artifacts, wire schemas, or obsolete
standard-library wrappers.

`fpas-bench` and `--std-lib` are intentionally omitted from the CLR toolchain.
They are not open gaps and do not require replacement commands or options.

The comparison uses Rust `main` at
[`0a4bc134e`](https://github.com/tobiasbick/functional-pascal/tree/0a4bc134ee71b5f922ebd0b1157ab55244bef657).
The current CLR owners are the [compiler CLI](../pascal/program-structure/cli.md),
[compiler API](../../src/Fpas.Compiler/Compilation.cs), and
[MSBuild contract](../build/msbuild.md). Recheck those sources before changing
an entry's status. The initial language/build baseline and its acceptance remain
separate in the [baseline comparison](../baseline-comparison.md).

## Current CLR support

The [CLI](../../src/Fpas.Cli/Compilation/CompilerCommand.cs) exposes only `compile`, plus root
and compile help and root `--version`. It accepts explicit sources, stdin, assembly references,
executable/library output, portable symbols, optional XML API documentation,
warning policy, JSON diagnostics, and JSON response files. Argument parsing and
generated help use `System.CommandLine` with the command definition in
[CompileCommand](../../src/Fpas.Cli/Compilation/CompileCommand.cs).

The project SDK already resolves dependencies and builds `.fpasproj` projects
and mixed `.slnx` solutions through MSBuild. Generated executable DLLs run through
`dotnet`. Incremental builds, Clean/Rebuild, portable PDBs, and XML documentation
have existing implementation and coverage. FPAS commands wrapping those
operations are missing.

The compiler API performs parsing, binding, and emission in one public operation.
It returns program bytes and diagnostics. A public analysis result suitable for
source queries, editor services, and refactorings is missing. A successful
check-only operation cannot currently return without assembly emission.

## Command inventory

The Rust `fpas` executable has twelve public command groups. Its
[argument parser](https://github.com/tobiasbick/functional-pascal/blob/0a4bc134ee71b5f922ebd0b1157ab55244bef657/crates/fpas-cli/src/cli_input/mode.rs)
and [help](https://github.com/tobiasbick/functional-pascal/blob/0a4bc134ee71b5f922ebd0b1157ab55244bef657/crates/fpas-cli/src/cli_input/help.rs)
define the comparison scope.

| Rust command | CLR gap and possible equivalent |
| --- | --- |
| `init project` | Missing program scaffold command. A CLR equivalent could create a `.fpasproj`, program source, and SDK configuration. |
| `init library` | Missing library scaffold command. A CLR equivalent could create a unit and a library `.fpasproj`. |
| `init workspace` | Missing solution scaffold command. A `.slnx` containing an application and a referenced library could replace the old workspace manifest. These three variants form one command group. |
| `build` | Missing FPAS frontend for the existing MSBuild build path. Old `.fpascu` and `.fpascp` outputs have CLR assembly equivalents. Native application packaging is a separate gap below. |
| `run` | Missing source/project/solution discovery, build-and-launch handling, and direct assembly execution. Arguments, standard streams, cancellation, working directory, and child exit status need a command contract. |
| `check` | Missing analysis-only command and compiler API mode. Project references may still need restore or dependency compilation to supply metadata. `--fix` has additional gaps below. |
| `debug` | Missing debug-session layer and DAP, JSONL, and one-shot batch frontends. Portable symbols already exist. See the debugger gaps below. |
| `test` | Missing FPAS test-program discovery and runner. Existing C# toolchain tests run with `dotnet test`; this does not provide the old FPAS test-program workflow. |
| `fmt` | Missing formatter for the current CLR language. Its parser/formatter needs to preserve comments and source meaning. Old Rust formatting rules cannot establish correctness for the new syntax. |
| `doc` | Missing declaration lookup and signature/documentation output. Source documentation and CLR metadata are available inputs; consuming referenced assemblies' XML documentation needs tooling. Existing XML generation is not a `doc` lookup command. |
| `query` | Missing semantic source-query frontend and common analysis service. All seven Rust subcommands are listed below. |
| `refactor` | Missing checked source-edit preparation and application. All five Rust subcommands are listed below. |
| `env --json` | Missing installation report covering compiler, FPAS SDK, .NET, and any available language-server/debugger tools. The old adjacent source-standard-library field has no direct CLR equivalent. |
| `stdlib stage` | Its VM-specific purpose does not carry over. CLR distribution uses SDK/support packages, assemblies, and NuGet dependencies. Reintroducing a `Std.*` catalog is outside the CLR design. |

Root and compile help support `--help`, `-h`, and `-?`; root `--version` reports
the compiler CLI version. Future commands need their own help and examples.
Command and ordinary option short aliases remain unselected.

## Project and solution inputs

The Rust project/workspace formats were `.fpasprj` and `.fpasworkspace`. CLR
projects use `.fpasproj` and `.slnx`. A common way to load evaluated project
sources, references, configurations, and output paths is missing from the CLI.
MSBuild already owns that information and dependency graph.

Possible inputs discussed for the CLI include a source file, project, solution,
and built assembly. For example, `fpas build App.slnx` and `fpas run App.slnx`
would provide the requested solution-level workflow. These commands do not
currently exist. Standalone sources also need framework/reference resolution
and temporary-output ownership beyond today's explicit `compile` inputs.

A solution can contain several executables and libraries. Solution execution
has no FPAS start-project rule yet. An explicit selection, a declared startup
project, or a unique executable are possible ways to resolve it. Multiple
candidates need an actionable ambiguity result. Mixed solutions can have a C#
or F# host consuming an FPAS library, so execution cannot assume that the FPAS
project is the application.

The syntax and selection order remain open. `fpas run App.slnx --project
Server/Server.fpasproj -- --port 4000` is an illustration, not an approved CLI
contract. Standard `dotnet run` is project-oriented; a solution-level FPAS
selection rule would be additional tooling.

## Source queries and refactorings

The Rust [query parser and help](https://github.com/tobiasbick/functional-pascal/blob/0a4bc134ee71b5f922ebd0b1157ab55244bef657/crates/fpas-cli/src/cli_input/query.rs)
expose these seven operations. Every CLR command is missing.

| Subcommand | Missing capability |
| --- | --- |
| `query find` | Search checked declarations in an explicit project/solution context. |
| `query inspect` | Return a declaration's signature, source implementation, and directly relevant types. |
| `query refs` | Find resolved uses and distinguish them from the declaration. |
| `query definition` | Resolve a use to its defining source identifier or external metadata identity. |
| `query type` | Report a checked expression or declaration's static type. |
| `query callers` | Describe incoming direct calls, callable-value uses, and unresolved dynamic targets. |
| `query callees` | Describe outgoing calls owned by one selected source body. |

These queries need reusable symbol identities, source ranges, reference indexes,
and accessible binding results. Analysis coverage, ambiguous names, coordinate
conventions, incomplete input, and dynamic-call limitations need explicit
results. Static inspection cannot promise every runtime target of a delegate,
interface, or virtual call.

The Rust [refactoring commands](https://github.com/tobiasbick/functional-pascal/blob/0a4bc134ee71b5f922ebd0b1157ab55244bef657/crates/fpas-cli/src/cli_input/refactor/help.rs)
expose these five operations. Every CLR command is missing.

| Subcommand | Missing capability |
| --- | --- |
| `refactor rename` | Resolve a symbol, check conflicts, and prepare changes to its declaration and uses. |
| `refactor add-parameter` | Check the declaration and affected calls, including modes, argument binding, and evaluation order. |
| `refactor extract-variable` | Extract an expression while preserving scope, evaluation order, and execution frequency. |
| `refactor organize-imports` | Remove safe unused imports and organize them while preserving aliases, comments, and initialization effects. |
| `refactor apply` | Revalidate a prepared plan against current inputs and apply its exact changes, with stale-plan and publication results. |

A shared analysis service is a possible common dependency for `check`, `doc`,
`query`, refactoring preparation, and a language server. That service is missing;
this inventory does not select its API or implementation. Source-edit plans and
revalidation are also absent. Adding these tools does not require inventing new
FPAS language syntax.

## Debugger and agent inspection

The Rust debugger has VM debug operations, a protocol-neutral engine, and DAP,
JSONL, and batch frontends. The agent-facing batch mode runs to a requested
breakpoint, runtime failure, or completion and emits one bounded JSON snapshot.
It includes stacks, bindings, globals, requested values, program output, and
explicit capture limits. The debugger itself does not need an LLM.

The CLR emitter already produces [portable debug information](../architecture/debug-information.md)
with source identities, statement sequence points, hidden generated code,
lexical locals, closure-associated locations, and async state-machine/yield/resume
metadata. Existing tests inspect symbols and source stack traces. They do not
establish a complete interactive FPAS debugging workflow with a selected CLR
debugger. Practical verification remains open in
[documentation gap D3](documentation-gaps.md#d3-concrete-debuggingprofiling-walkthroughs).

Missing capabilities include:

- A CLR debugger backend integration and common session state for launch,
  pause/resume, stepping, breakpoints, frames, scopes, values, and exceptions.
- A DAP frontend or integration suitable for FPAS editors. An existing CoreCLR
  adapter such as [NetCoreDbg](https://github.com/Samsung/netcoredbg) is a
  candidate to evaluate. Compatibility with generated FPAS assemblies, .NET 10,
  synchronous stepping, and real async suspension has not been established here.
- A JSONL session interface and a bounded one-shot JSON snapshot command for
  agents. Output separation, deadlines, value limits, unavailable data, and
  inspection-handle lifetimes lack a CLR FPAS contract.
- FPAS watch-expression and conditional-breakpoint semantics. A debugger's C#
  evaluator does not implement Pascal operators, aliases, or FPAS name lookup.
- Source-oriented presentation of captures, domains, unions, and generated
  async storage. PDB emission alone does not supply every frontend convention.
- A defined policy for expression evaluation and mutation. The Rust VM's
  detached-value evaluation cannot simply be assumed for arbitrary .NET calls
  or property getters, which can execute code and change live state.
- A distinction between OS threads, CLR tasks/async continuations, and the old
  VM's stable task identities and all-stop scheduler.

VM task controls, forced return, selected-frame restart, data/assignment
tracepoints, recording, and compatible image reload/rollback have no established
CLR equivalents in this port. Each depends on backend capabilities or additional
compiler/runtime tooling. Reverse execution and replay were not supported by the
old debugger either.

## Intentionally omitted capabilities

`--std-lib` is intentionally omitted. The CLR port has no interchangeable
`Std.*` source library. `TargetFramework`, the FPAS SDK version, and ordinary
package, project, and assembly references select the platform and dependencies.
No equivalent override option is needed. This is not an open CLI gap.

`fpas-bench` is intentionally omitted, including its `run`, `save`, `compare`,
`record`, and `native` modes. No port or replacement FPAS benchmark CLI is
needed. Compiler performance, emitted CIL, and allocations can still be measured
with existing .NET tools when needed. This is not an open CLI gap.

For completeness, the Rust benchmark tool's native drivers were
`language-service`, `language-service-project`, `compiler-lowering`,
`project-build`, `unit-artifact`, `program-artifact`, `http-body`, and
`error-values`. These omitted development workloads do not create CLR CLI gaps.

## Options and behavior gaps

| Rust capability | CLR status or open question |
| --- | --- |
| `init --path`, library `--unit`, `--dry-run`, `--report json` | Missing scaffold path/name selection, file-plan preview, and structured results. The old command preserved existing files and accepted an unchanged repeated invocation. CLR scaffold behavior has no contract yet. |
| `build --executable`, `--name` | Publishing/apphost support is outside the current FPAS SDK scope. .NET self-contained and single-file publishing are possible equivalents. NativeAOT is a separate compatibility question. |
| `check --fix` | Diagnostics currently carry code, message, location, and severity, without machine-applicable edit data. Fix production, edit ownership, bounded rechecking, and publication are missing. |
| Query selectors, `--symbol`, `--target-ref`, `--context`, `--limit`, `--timeout` | Missing project-aware symbol selection, bounded analysis, durable target references, and completeness results. |
| Refactor `--name`, `--type`, `--mode`, `--argument`, `--range`, `--out`, `--plan` | Missing typed fragment/range validation, source-edit plan output, and explicit plan application. Context selection and analysis deadlines are also absent. |
| `run --profile`, `debug --profile` | .NET tracing/profiling can supply observations, but there is no FPAS collection/summary command. VM instruction, scheduler, and runtime-phase counters do not carry over unchanged. |
| `run --failure-dump` | No structured FPAS runtime-failure report command. Exception/source stacks are a starting point; local values from arbitrary frames need debugger or dump analysis. A generic process launcher cannot recover them from stderr. |
| `test --list`, `--filter`, `--file`, `--jobs`, `--timeout`, `--fail-fast`, `--show-output`, `--report json` | Missing discovery/selection and isolated CLR test-program execution with structured results, output capture, deadlines, and process cleanup. |
| Test hooks, assertions, skip/`--strict`, scratch directories, and golden output | The old `*_test.fpas` program convention and `Std.Test` behavior need an explicit CLR counterpart or test-framework integration. A source test suite cannot simply put multiple program entries in one ordinary executable project. |
| Test/debug `--script` and interactive terminal input | Redirected standard input/output is available to CLR processes. The old hosted-console events, virtual screen, and terminal assertions have no equivalent in this port. |
| `fmt --check`, `--list`, `--stdout` | Missing formatter modes for validation, changed-file listing, and output without source replacement. |
| `--no-contracts` | No disable switch exists. Contracts execute in Debug and Release. Optional omission would require an explicit policy decision, not merely parsing a flag. |
| `--gc-cell-threshold`, `--gc-mutation-threshold`, `--gc-poll-interval` | VM collection tuning has no matching CLR GC flags. |
| Debug `--schedule-seed` and `--instruction-limit` | Seeded VM scheduling, virtual time, and exact dispatched-bytecode limits do not apply to ordinary CLR execution. Process deadlines are a different guarantee. |
| Debug `--trace`, `--trace-limit`, `--task`, `--frame`, `--print`, `--break` | Missing commands; inspection selectors and trace coverage depend on the chosen CLR debugger integration. |
| Debug `--protocol dap/jsonl`, `--commands`, `--source-root`, `--timeout`, `--output-limit` | Missing protocol/session frontends, scripted JSONL input, symbol/source matching policy, and bounded lifecycle. Old `.fpascp` source-root validation is not the CLR assembly/PDB contract. |
| JSON diagnostics and reports | `compile --diagnostics json` already returns one structured result. Other command reports, consistent failure output, and coverage/truncation fields are missing. Rust JSONL diagnostics and schemas are not automatically supported. |
| Help, version, response files, and program arguments after `--` | Generated root/compile help, help aliases, root version output, JSON compile response files, and literal compile source paths after `--` exist. Program-argument forwarding and behavior for future commands remain missing. |

.NET provides [single-file deployment](https://learn.microsoft.com/en-us/dotnet/core/deploying/single-file/overview)
and [diagnostic tools](https://learn.microsoft.com/en-us/dotnet/core/diagnostics/tools-overview).
Their availability does not establish FPAS SDK publishing support or the old
bounded JSON report behavior. See the current
[debugging](../pascal/tools/debugging.md) and
[profiling](../pascal/tools/profiling.md) guides for implemented scope.

## CLI argument library

[System.CommandLine](https://learn.microsoft.com/en-us/dotnet/standard/commandline/)
2.0.14 is the CLI's argument library, pinned in the central package versions.
It supplies the command tree, typed argument/option binding, defaults, allowed
values, generated help, help aliases, and version output. Compile-specific
validation and output-path protection remain in the CLI. Parsed compile requests
are separate from source loading and compilation.

JSON-array response files retain their existing contract: they replace all
inline compile arguments and cannot nest. The library's `@file` response syntax
is disabled. Literal sources after `--`, stdin, spaces in paths, and repeatable
references are covered by CLI execution tests. Argument errors retain exit code
2 and `FP0001`; JSON diagnostics emit one result without parser/help text.
The project SDK packages the parser assembly and its localized resources with
the compiler.

This integration does not supply the missing MSBuild commands, semantic analysis,
debugger, or report schemas. Future commands still need explicit forwarding,
cancellation, completion, and protocol-stream behavior; distribution/AOT
acceptance remains in [distribution gaps](dist-gap.md).

## Related executable tools

These Rust binaries are outside the twelve `fpas` command groups:

| Tool | CLR gap or equivalent |
| --- | --- |
| `fpas-lsp` | Language-server and editor/project integration gaps are tracked in [editor gaps](editor-gap.md). The same analysis service could support CLI queries and LSP. The Rust executable uses stdio and has help/version options; it was not `fpas lsp`. |
| `fpas-runner` | The VM bundle host is superseded by the .NET runtime or a published application host. Preserving its VM image loader has no CLR purpose. |
| `fpas-bench` | Intentionally omitted, including all modes and native drivers. No port or replacement FPAS benchmark CLI is needed. This is not an open gap. |

Private query/debug/test worker entry points are internal process protocols,
not user-facing commands.

There was no separate `fpas profile`, `fpas clean`, `fpas restore`, or
`fpas publish` in this Rust command inventory. Profiling used options. CLR
wrappers for clean/restore/publish would be new convenience commands and remain
unselected. [Documentation gap D1](documentation-gaps.md#d1-reproducible-benchmarks)
concerns reproducible measurement procedures using .NET tooling. It does not
request a port or replacement of `fpas-bench`.

## Documentation and acceptance with CLI work

Deliver the related documentation work alongside the corresponding CLI
capabilities:

- [D3](documentation-gaps.md#d3-concrete-debuggingprofiling-walkthroughs) accompanies
  debugger and diagnostic implementation, including verified source breakpoints,
  parameter/local inspection, captures and async continuations, profiling, and
  controlled failure-dump workflows.
- [D1](documentation-gaps.md#d1-reproducible-benchmarks) accompanies the relevant
  profiling and diagnostic work with reproducible workloads, result checks, and
  comparable measurements using .NET tools. `fpas-bench` remains intentionally
  omitted; this does not require a replacement benchmark command.

D1 and D3 remain separate, open acceptance items until their respective documented
workflows have passed acceptance on both Windows and Linux. Implementing a CLI
capability alone does not close either item. Their detailed criteria remain in
the [documentation gaps](documentation-gaps.md). This coordination does not select
additional commands, dependencies, or an implementation order.

## Keeping the inventory current

An entry can be removed or narrowed when its owning implementation and relevant
verification establish the capability. Update the ordinary CLI/tooling reference
when a command becomes usable. Existing symbol emission, .NET tools, or MSBuild
targets alone do not establish a complete FPAS command.

This inventory does not add missing-baseline claims to
[language gaps](language-gaps.md), schedule work, or authorize changes to source
syntax. Future decisions can select, revise, or discard open counterparts.
Intentionally omitted capabilities do not count as unfinished work.
