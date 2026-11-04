# Compiler and source analysis CLI

The current CLI compiles FPAS sources into ordinary CLR libraries or executable
DLLs. Project builds invoke the same compiler through the SDK. The `debug`
command hosts [MCP agent debugging](../tools/debugging.md#agent-debugging-with-mcp)
using a separately installed NetCoreDbg; inspect it with `fpas debug --help`.
The CLI uses System.CommandLine for argument binding and generated help.

## Check a project or solution

`fpas check Game/Game.fpasproj` validates evaluated on-disk sources without
publishing its own DLL, PDB, runtime configuration or XML documentation.
Use an explicit `.fpasproj` or `.slnx`; source-level imports do not select projects.
The selected SDK/compiler must match the CLI, and support the analysis targets.

`fpas check Game.slnx --configuration Release --diagnostics json` produces one
JSON document with project states (`checked`, `invalid`, `blocked`), diagnostics,
coverage, snapshot identity and preparation side effects. Independent projects
are checked even if another project fails.

Preparation permits ordinary restore and dependency builds; those can write
bin/obj artifacts and execute MSBuild tasks. `--no-restore` skips only restore.
The checked root's existing outputs and successful-build fingerprint are
preserved. Documentation/warning policy comes from the evaluated project.
Custom source generation uses the SDK's shared
[FpasPrepareCompilation target](../../build/msbuild.md#compiler-and-build-contract).

Default diagnostics and the completion summary go to stderr. Check exits with
0 for success, 1 for source/setup/timeout errors, 2 for invalid arguments or
ambiguous project contexts, and 130 for cancellation. `--timeout` covers the
whole operation, defaults to 120 seconds, and accepts 1..3600. Cancellation stops
owned build processes. Changed inputs invalidate the result before publication.
Linux is integration-tested; Windows acceptance remains open.

## Query checked sources

Use an explicit context and source position:

```sh
fpas check examples/units/Units.fpasproj
fpas query definition examples/units/Units.fpasproj --file examples/units/main.fpas --line 6 --column 29
fpas query type examples/units/Units.fpasproj --file examples/units/main.fpas --line 7 --column 21
fpas query refs examples/units/Units.fpasproj --file examples/units/lib/calculations.fpas --line 8 --column 17 --limit 100
```

Query output is one JSON document, including argument/setup failures. Its status
is ok, not_found, failed or cancelled. It contains context/configuration, snapshot
identity, selected project/span, diagnostics, analyzed/searched/blocked coverage,
preparation artifacts, binary boundaries and the typed result. No query executes
the application, its initialization, imported attributes or getters.

Paths resolve against the invocation directory. Lines/columns count original
UTF-16 code units from one; tabs count as one and CR/LF/CRLF remain intact.
Definition and refs select the containing identifier token, excluding its end
position, whitespace and comments. Type selects the smallest indexed expression
or type extent and returns checked/declared types, generic substitutions,
nullability and callable signatures. Namespaces return not_found/not_typed.

Definition follows the bound overload or static interface contract. Referenced
FPAS declarations map through verified dependency builds; other .NET definitions
carry exact metadata identities without a source path. Implicit args/self have
no explicit declaration and return not_found/implicit_declaration.

Refs excludes the declaration, comments, strings and generated storage. Results
are deterministic and identify read, write, read_write, argument, type or namespace
roles. The search covers analyzed FPAS projects in the explicit context; it does
not promise unknown consumers, C#/F# source uses or runtime dispatch targets.
Complete zero references differs from failed/incomplete analysis. The default
limit is 100, range 1..1000; omitted results set truncated independently of coverage.

Use `--project <owning.fpasproj>` when a source is included in several compilations.
An ambiguous selection returns candidate project paths. Selection does not narrow
the refs search scope. Configuration, no-restore, timeout and exit codes match
check. Query responses are bounded to 4 MiB; oversized output returns result_limit
as complete JSON. Source/input changes invalidate results before publication.

## Inspect and invoke the compiler

After building the toolchain, inspect the actual supported arguments:

```sh
dotnet src/Fpas.Cli/bin/Debug/net10.0/Fpas.Cli.dll compile --help
```

No arguments show root help. Root and compile help accept `--help`, `-h`, and
`-?`. Root `--version` prints the CLI's assembly informational version.
Help usage and examples always use the public command name `fpas`, including
when the compiler is invoked directly through `dotnet`.

For standalone compilation, set FPAS_REFERENCE_DIR to the installed net10.0
reference assembly directory as described by the
[development workflow](../../build/development.md#direct-compiler-invocation).

```sh
dotnet src/Fpas.Cli/bin/Debug/net10.0/Fpas.Cli.dll compile examples/hello/main.fpas --output .temp-data/hello/Hello.dll --reference-dir "$FPAS_REFERENCE_DIR" --diagnostics json
dotnet .temp-data/hello/Hello.dll
```

Supply multiple source paths for a program with units, or select --target library
for units-only output. One source path may be - to read strict UTF-8 from stdin.
Repeat --reference for assemblies or --reference-dir for a directory's DLLs.
Each occurrence takes one value. Options accept both `--output path.dll` and
`--output=path.dll`. Place source paths starting with `-` after `--`, following
all options. A lone `-` remains the stdin source.
File inputs are resolved to absolute paths before compilation, so diagnostics
and portable PDB documents identify the source independently of the program's
runtime working directory. Stdin retains the virtual source name `<stdin>`;
the CLI does not create a saved source file for it.
The compiler receives explicit metadata inputs, including System.Runtime; it
does not bind whichever assemblies happen to be loaded in its process.

## Options and diagnostics

| Option | Behavior |
| --- | --- |
| --target exe or library | Validate program/unit count and choose output kind |
| --output path.dll | Name the managed assembly |
| --debug portable or none | Emit matching portable symbols or disable them |
| --optimize | Allow JIT optimization, independently of portable symbols |
| --generate-documentation | Emit a conventional XML sidecar |
| --documentation path.xml | Select an explicit XML documentation output |
| --diagnostics json | Emit a structured success/output/diagnostics result |
| --response-file path.json | Read a JSON string array of compile arguments |
| Warning policy options | Promote, exclude, or suppress documentation warnings |

Response files replace inline compile arguments; nesting is unsupported. Paths
are structured arguments, not shell-expanded command text. The parser's `@file`
response syntax is disabled; `@` is a literal character in source paths.
Default diagnostics
go to stderr; successful output paths go to stdout. JSON mode emits one result
object with diagnostic codes and source locations. See [diagnostics](../tools/diagnostics.md).
Argument errors use `FP0001` and exit code 2, compilation/I/O failures use exit
code 1, and success/help/version use exit code 0. JSON argument errors contain
one result object without additional parser output.

Failed compilation preserves existing outputs. Publication stages the assembly,
symbols, runtime configuration, and requested XML together and restores previous
outputs if replacement fails. Parallel invocations must select distinct output
paths. Source and reference inputs must differ from output paths. Reference
assemblies are protected from replacement or deletion at the assembly, PDB,
runtime configuration, and requested XML paths, including stale sidecars removed
by --debug none or --target library.

Direct deployment must include non-framework dependencies; prefer the
project SDK for normal applications.

## See also

- [Projects](projects.md)
- [Compiled programs](compiled-programs.md)
- [Compiler API and output publication](../../architecture/compiler.md#compiler-api-and-output)
- [Development workflow](../../build/development.md)
