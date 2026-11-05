# Create, build and run

Use explicit paths. `fpas init` creates files; `fpas build` invokes the selected
project SDK through MSBuild; `fpas run` builds and launches the selected managed
executable. FPAS source imports do not select projects or download dependencies.

## Create files

```sh
fpas init project Hello --name Hello --sdk-feed sdk/feed
fpas init library Math --name Demo.Math --sdk-feed sdk/feed
fpas init solution Demo --project Hello/Hello.fpasproj --project Math/Demo.Math.fpasproj
```

Each path names a destination directory. A project creates `Name.fpasproj` and
`main.fpas`; a library creates `Name.fpasproj` and `Name.fpas`. Templates target
net10.0 and explicitly select Exe or Library output. A solution creates
`Name.slnx`, with FPAS project-type configuration and any repeated `--project`
members. Members must already exist and can be FPAS, C# or F# projects. Without
members the solution is an empty container; it does not create an application.

The name defaults to the destination directory name. `--name` accepts a qualified
Pascal identifier and avoids nonportable device filenames. Keyword components
are escaped in generated source. Paths can contain spaces and Unicode.

`--sdk-version` selects an explicit package version. Its default is the containing
SDK package version when running packaged tools, otherwise the CLI's built-in SDK
version. `--sdk-feed` creates a local NuGet.Config with the specified directory or
HTTPS feed and nuget.org. Local feed paths are written relative to that file.
Without the option, existing NuGet configuration applies. No FPAS SDK package is
publicly published; prepare a feed or cache as described in the
[standalone SDK guide](../../build/standalone-sdk.md). Solutions do not select an
SDK/feed; their projects own those settings.

`--dry-run` previews the plan without creating directories or files.
`--report json` includes every planned file's path, state and exact content.
Identical existing files remain unchanged, including their timestamps. Any
differing existing file/directory is a conflict and prevents all planned writes.
Creation uses exclusive new files; an I/O failure rolls back files created by
that operation. Unrelated files are preserved.

## Build

```sh
fpas build examples/hello/Hello.fpasproj
fpas build Demo/Demo.slnx --configuration Release --report json
```

Build accepts `.fpasproj`, `.csproj`, `.fsproj` and `.slnx`. It uses the normal
MSBuild graph, project settings, incremental outputs and solution configuration
mappings. Configuration defaults to Debug. Restore is enabled; `--no-restore`
requires existing restored assets and does not suppress compilation.

A standalone `.fpas` input selects exactly that source as an executable. It uses
an owned generated project under `obj/fpas-cli/<source-name>/` and writes normal
`bin/<configuration>/net10.0/` outputs beside the source. Build and run reuse this
project/assets. `--sdk-version` and `--sdk-feed` select its SDK/feed; existing
parent build/NuGet settings otherwise apply. Multi-source units and additional
references belong in an explicit project. Normal project inputs reject these
source-only SDK options.

Build defaults to a 120-second overall deadline; `--timeout` accepts 1..3600.
Build workers and compiler servers are disabled for owned-process cleanup;
MSBuild retains its default build parallelism. Captured output is bounded to
1 MiB across stdout and stderr. An exceeded limit is a failure, even if MSBuild
has emitted artifacts. Failed FPAS compilation preserves prior compiler outputs.

## Run

```sh
fpas run examples/hello/Hello.fpasproj --no-build -- first --second
fpas run Demo/Demo.slnx --project Hello/Hello.fpasproj --configuration Release
fpas run Hello/bin/Debug/net10.0/Hello.dll -- argument
```

Run accepts the same source/project/solution inputs, plus existing managed DLLs
with adjacent runtimeconfig.json files. A DLL is launched directly. An explicit
project must be executable. For a solution, exactly one executable among its
included build projects is selected automatically; ambiguity reports candidate
paths and requires `--project`. A C# or F# host can be the executable. Selection
honors solution configuration/platform mappings. An outside or excluded project
cannot be selected. Selection is resolved before building; the selected solution
is then built with its ordinary dependency graph.

By default run restores/builds before launching. A failed build never falls back
to an older executable. `--no-restore` only skips restore; `--no-build` explicitly
runs existing project output and implies no restore. Standalone sources reuse
their generated project's output with `--no-build`; they cannot use `--project`.
DLL inputs do not accept project/
restore/build selection flags.

Pass program arguments after `--`; their values are forwarded verbatim without
shell interpretation. The working directory defaults to the caller's current
directory for every input form; `--working-directory` selects another existing
directory. Runtime assembly lookup still belongs to .NET and the output's
dependency files.

Normal mode inherits stdin/stdout/stderr, preserving piped input and terminal
handles. Build messages go to stderr. JSON mode inherits stdin and captures
program stdout/stderr as UTF-8 text, with a shared 1 MiB limit; no target output
is mixed into protocol stdout. An exceeded output limit is explicit. Each run
owns its process group/job and terminates/joins remaining descendants when the
main process exits, on cancellation or on a deadline.

Run has no implicit deadline. `--timeout` accepts 0..3600 seconds for the entire
operation; zero disables the deadline. Cancellation remains available.

## Reports and exit codes

All three commands support `--report json`: one stdout document for success or
failure, including argument failures. Fields identify command, status and context;
available file plans, selected target, build output, program result and error are
separate fields. Build/program capture reports exit codes and truncation.

| Exit | Meaning |
| --- | --- |
| 0 | Successful init/build or program exit 0 |
| 1 | Creation/build/setup failure, deadline or captured output limit |
| 2 | Invalid arguments, conflicting executable selection or non-executable input |
| 130 | User cancellation after owned-process cleanup |
| Program exit | Run preserves the program's exit code, including nonzero values |

A run report distinguishes a program exit from a host failure even when numeric
codes overlap. Init file conflicts use exit 1 and `file_conflict`. These commands
do not publish applications, choose AOT/apphosts, discover tests, or format sources.
Native Windows console/job acceptance remains in the [CLI gaps](../../future/cli-gap.md).

## See also

- [Project shape and dependencies](projects.md)
- [Compiler and source analysis CLI](cli.md)
- [SDK bootstrap and development](../../build/development.md)
