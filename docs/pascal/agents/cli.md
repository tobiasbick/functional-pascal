# FPAS command-line reference for agents

Use `fpas --help` for top-level commands, `fpas COMMAND --help` for options
and subcommands, and `fpas --version` for the installed version. Help also accepts
`-h` and `-?`. Use explicit `.fpasproj` or `.slnx` paths for project
operations. The [development workflow](../../build/development.md) shows how to
invoke the same CLI from a source checkout. Keep scratch outputs under `.temp-data/`.

## Projects, deployment, and testing

| Command | Use |
| --- | --- |
| `fpas init` | Select a project template. Preview file creation with `--dry-run --report json`. |
| `fpas init project` | Create a runnable project. Use `--name` for the FPAS name or `--sdk-feed` for a local feed. |
| `fpas init library` | Create an FPAS library project. |
| `fpas init test` | Create a test project for `xunit`, `mstest` or `nunit`. Use `--framework mstest --runner mtp` for a self-hosted application. |
| `fpas init solution` | Create a solution and include existing projects with repeated `--project`. |
| `fpas restore` | Restore packages for an explicit project or solution through the selected SDK. |
| `fpas build` | Build a source, project or solution through MSBuild. |
| `fpas run` | Build and start a source, project or solution, or run a built DLL. Pass program arguments after `--`. |
| `fpas target` | Report the selected executable and paths as JSON without restoring or building. Select an ambiguous solution target with `--project`. |
| `fpas clean` | Run SDK clean for the selected project or solution and configuration. |
| `fpas publish` | Deliver an executable or runtime files to `--output`. Native AOT is the default. Other modes include `single-file`, `self-contained`, `apphost` and `framework-dependent`. |
| `fpas test` | Build and run .NET tests through VSTest or MTP. Discover with `--list`, reuse outputs with `--no-build`, and read JSON and TRX results. |
| `fpas terminal` | Select a private terminal workflow. |
| `fpas terminal test` | Run a JSON script of literal send, expect, resize and exit steps in a private PTY or ConPTY. Read the JSON report. |
| `fpas sdk` | Select the embedded SDK export operation. |
| `fpas sdk export` | Write the matching SDK package to a local NuGet feed. Identical packages are kept. An explicit `--force` replaces a different package at the same path. |
| `fpas licenses` | Print bundled debugger and .NET runtime notices. |

`fpas run --report json` captures output and inherits stdin. Use `--timeout` for a
bounded run. `--failure-report FILE` writes the first unhandled exception with the
debugger's stack and available locals to a new file, then resumes the same
process and preserves its exit code. Keep its DLL, portable PDB and matching sources together.
[Project commands](../program-structure/project-commands.md) · [Run failure reports](../program-structure/project-commands.md#save-the-original-failure)

[Publishing modes and prerequisites](../../build/publishing.md) · [Native testing](../tools/testing.md) · [Terminal scripts](../tools/terminal-testing.md) · [SDK export and distribution](../../build/distribution.md#export-the-matching-sdk)

## Compile, analyze, format, and discover APIs

| Command | Use |
| --- | --- |
| `fpas compile` | Compile explicit sources to a CLR DLL. Supply `--output`, framework metadata via `--reference-dir`, and extra assemblies via repeated `--reference`. Use `--target library` for units-only output. |
| `fpas check` | Validate an evaluated project or solution without analysis-owned output. Use `--diagnostics json`, `--fix --dry-run` to preview corrections and `--fix` to apply one checked batch. |
| `fpas fmt` | Format sources, directories, projects or solutions. `--check --list` reports affected paths and `--stdout` previews one source. |
| `fpas doc` | Look up FPAS and referenced CLR APIs with `--name`. Use `--usage` with the selected XML documentation ID for a compiler-checked FPAS template. |
| `fpas query` | Select a checked source query. Results include a snapshot identity and coverage. |
| `fpas query definition` | Resolve a symbol to its declaration. |
| `fpas query type` | Get the checked type at a source position. |
| `fpas query refs` | Find checked references across the declared FPAS scope. |
| `fpas query find` | Search declarations by `--name`, with `--match exact\|prefix\|contains` and repeatable `--kind`. |
| `fpas query inspect` | Read the declaration, attached docs and directly related types for a symbol. |
| `fpas query callees` | List outgoing operations for the resolved callable, or the enclosing body with `--body`. |
| `fpas query callers` | Find incoming calls and callable references. Use `--body` for an enclosing routine or lambda. |

Use `--file`, `--line` and `--column` for positional queries. Lines and UTF-16
columns are one-based. `--project` selects the owner of an ambiguous linked
source. With paged queries, use `--limit` and `--offset` and combine pages only
when their `snapshotId` matches. Check coverage and truncation before relying on
the result as exhaustive. `check` and `query` can restore or build dependencies.
`--no-restore` skips only restore. [Compiler and source query contracts](../program-structure/cli.md)

Use an `M:` ID for a method or constructor, `P:` for a property or indexer,
and `F:` for a field. `doc --usage` reports required inputs or explains why the
template is unavailable. An instance member takes the receiver first. Use
`--access write` for a property or field assignment and repeat `--type-arg`
for enclosing generic types and then the method. [API lookup and usage](../tools/api-documentation.md)

[Diagnostics and corrections](../tools/diagnostics.md) · [Formatting](../tools/fmt-style.md)

## Checked refactorings

| Command | Use |
| --- | --- |
| `fpas refactor` | Select an operation that prepares a JSON edit plan for review. |
| `fpas refactor rename` | Rename a resolved source symbol with `--new-name`, including checked references and connected source interface members. |
| `fpas refactor add-parameter` | Add a value parameter and update direct calls. Supply `--name`, `--type` and `--argument`. |
| `fpas refactor remove-parameter` | Remove an unused value parameter and arguments proven safe to omit. |
| `fpas refactor change-signature` | Change a parameter with `--type` or `--mode`. Supply replacements with repeated `--argument [FILE:]LINE:COLUMN=EXPRESSION`. Or select the routine and reorder with `--order Second,First`. |
| `fpas refactor implement-protocol` | Select a record and pass `--protocol TYPE`. Repeat `--member MEMBER=STATEMENTS` (initializer for a getter) or `--forward MEMBER=METHOD`. Missing behavior reports member keys. |
| `fpas refactor name-arguments` | Insert labels at a checked call with repeated input types while preserving evaluation order. |
| `fpas refactor extract-variable` | Move a selected expression into a typed local `const`. Give the exclusive end with `--end-line` and `--end-column`, and the new `--name`. |
| `fpas refactor extract-routine` | Move consecutive statements into a nested helper. Include the final semicolon and give `--end-line`, `--end-column` and `--name`. |
| `fpas refactor inline-variable` | Select a local declaration or read. Replace literal reads or move a single initializer to the next statement when evaluation order is safe. |
| `fpas refactor organize-imports` | Use `--file` to remove unused CLR aliases when all owners agree. Preserve source-unit imports, comments and entry order. |
| `fpas refactor apply` | Recheck the snapshot and regenerate the plan before publishing its edits. |

Prepare with `--plan FILE.json` and inspect the edits. Positional selection uses
the same coordinates as `query`. Apply with the original project or solution,
configuration and plan path. Use `--report json` for machine-readable results.
The command refuses conflicting or stale plans and verifies the written sources.
[Refactoring options and examples](../tools/refactoring.md)

For public parameter changes, use `--closed-context` with a context containing
the declaring project and every consumer. Known foreign project and dependent
binary boundaries block the change. [Public signatures](../tools/refactoring.md#public-signatures-across-projects)

## Runtime inspection and toolchain

| Command | Use |
| --- | --- |
| `fpas debug` | Debug a built CLR DLL. Use `--break` for a one-shot stop, `--tracepoint` with `--max-hits` for repeated observations, or `--protocol dap` for an editor session. |
| `fpas diagnose` | Select runtime failure or dump inspection. |
| `fpas diagnose failure` | Run a built DLL and report an unhandled managed exception with frames, source and passive bindings as JSON. |
| `fpas diagnose dump` | Capture a heap dump from an explicit live .NET `--process-id` to a new `--output` file. |
| `fpas diagnose inspect-dump` | Inspect a saved dump with bounded, repeatable SOS `--command` requests. |
| `fpas profile` | Select a profiling workflow for a built target. |
| `fpas profile setup` | Install pinned Microsoft diagnostic tools in an isolated directory. |
| `fpas profile collect` | Record a target in `threads`, `gc` or `counters` mode to a new artifact. |
| `fpas profile summary` | Decode EventPipe counts, per-thread samples and GC events from a saved trace. |
| `fpas env` | Inspect the installed CLI, SDK/cache, .NET host, debugger and language server. Use `--json` for the report and `--directory` for SDK selection in another directory. |
| `fpas lsp` | Start a stdio language server. Supply an absolute context path or file URI in `initializationOptions.context`. LSP coordinates are zero-based UTF-16. |
| `fpas schema` | Export the named JSON Schema to stdout. Available names are `operation`, `terminal-script`, `project`, `compile`, `check`, `query`, `test`, `debug`, `format`, `doc`, `refactor` and `environment`. |

Keep matching DLLs, sources and portable PDBs for source debugging. Breakpoint
paths resolve against `--cwd` or the DLL's directory. Batch inspection is passive.
Keep raw traces and dumps private. Measure unprofiled execution for timings.
[Batch debugging](../tools/batch-debugging.md) · [Diagnostic CLI](../tools/cli-diagnostics.md) · [Measurements](../tools/benchmarks.md)

[Environment](../tools/environment.md) · [Language server](../tools/language-server.md) · [Report schemas](../tools/report-schemas.md)
