# CLI

The `fpas` command-line interface creates scaffolds, builds artifacts, discovers
projects, type-checks, runs programs, and executes test bundles.

## Usage

- `fpas` (no arguments) — prints usage to stdout and exits successfully.
- `fpas init <project | library | workspace> <name>` — create a formatted,
  immediately checkable scaffold without interactive prompts. `--path <dir>`
  selects the target, `--dry-run` previews without writing, and `--report json`
  produces machine-readable stdout. Libraries also accept `--unit <name>`.
- `fpas build [<path>]` — build a `.fpasprj` or `.fpasworkspace`. With no path,
  discovers a workspace first and otherwise a single project in the current
  directory. Program projects produce or reuse `<project.name>.fpascp`;
  libraries build `.fpascu` sidecars; workspaces process every member.
- `fpas build --executable [--name <name>] [<path>]` — build exactly one
  program and bundle it with the native FPAS runner for the current host.
  Projects default to `project.name`; workspaces default to `workspace.name`.
  Windows outputs `<name>.exe`; Linux outputs executable `<name>`. There is no
  cross-compilation.
- `fpas run` — discovers what to run in the current directory:
  - If a `.fpasworkspace` file exists: runs the sole `kind = "program"` member; errors when there are zero or multiple program members.
  - Otherwise searches for a `.fpasprj` file and requires exactly one match.
- `fpas run <path>` — detects input type by extension:
  - `.fpas` — runs as a single source file with a `program` declaration (no project needed).
  - `.fpasprj` — loads the program project, produces or reuses
    `<project.name>.fpascp`, and runs that image.
  - `.fpasworkspace` — runs its sole `kind = "program"` member through the same
    project-artifact path; errors when there are zero or multiple programs.
  - `.fpascp` — validates and runs the compiled program directly without
    loading sources, manifests, compiled units, or the source standard library.
  - Other extensions — error.
- `fpas run` with more than one positional path argument — usage error.
- `fpas check [<path>]` — type-check a `.fpas`, directory of `.fpas` files, `.fpasprj`, or `.fpasworkspace` without running. With no path, discovers `.fpasworkspace` or `.fpasprj` in the current directory.
- `fpas test [<path>]` — run `*_test.fpas` programs and print a pass/fail/skip summary. With no path, discovers a workspace or `.fpasprj` like `fpas check`. Flags: `--list`, `--fail-fast`, `--strict` (exit `1` when any test called `Skip`), `--filter <pattern>`, `--report json`, `--timeout <secs>` (default: `300`), `--jobs <n>` (`0` = available CPU parallelism), `--script <path>`. Sidecars beside each test file (all optional): `<test>.script.toml` (scripted input), `<test>.expect.stdout`, and `<test>.expect.screen` (TUI). See [`Std.Test`](../std/testing/test.md). `--list` and `--report json` write results to stdout; progress lines stay on stderr. A write failure on contracted stdout or summary output returns nonzero instead of reporting success. Each test's timeout budget starts before its isolated worker is spawned and covers worker preparation and VM execution. Tests run in a terminable process tree, so blocking startup, VM, or host calls cannot extend the timeout indefinitely.
- `fpas debug [<path>] --protocol <jsonl | dap>` — run a source, program
  project, workspace, or verified compiled image under the source debugger;
  see [Source debugger](../tools/debugger.md).
- `fpas fmt [<path> ...]` — format source files, directories, projects, or
  workspaces. A program project includes its `project.main` source as well as
  unit sources. `--check` reports formatting drift without changing files;
  `--list` prints changed paths with `--check`. `--stdout <file.fpas>` formats
  one source file to stdout without modifying it and cannot be combined with
  `--check`.
- `fpas -h` / `fpas --help` — prints the short command overview to stdout and exits successfully.
- `fpas init --help`, `fpas init <kind> --help`, `fpas build --help`,
  `fpas run --help`, `fpas check --help`,
  `fpas test --help`, `fpas debug --help`, and `fpas fmt --help` — print focused command help with
  valid examples and exit successfully.
- `fpas -V` / `fpas --version` — prints the compiler version to stdout and exits successfully.
- `fpas build --std-lib <directory> …`, `fpas run --std-lib <directory> …`,
  `fpas check --std-lib <directory> …`, and
  `fpas test --std-lib <directory> …` — replace the complete
  implementation-owned source standard library for that invocation. The
  directory must contain `stdlib.fpasprj`. Without this option, `fpas` loads
  `lib` beside its executable.

Program arguments after `--` require `fpas run` or `fpas debug` and are visible
through `Std.Args` when running programs.

## Command help

Start with `fpas --help` to discover commands, then request the relevant command's
help for its options and examples. This keeps terminal output concise and makes
copy-pasteable invocations available where they are needed:

```sh
fpas init --help
fpas init library --help
fpas build --help
fpas run --help
fpas check --help
fpas test --help
fpas fmt --help
```

## Initializing a scaffold

```sh
fpas init project hello
fpas init library greet --unit Demo.Greet
fpas init workspace acme-suite
fpas init workspace acme-suite --dry-run --report json
```

The default target is a new `<name>` directory. Existing files are never
overwritten; an identical retry succeeds with `status: unchanged`, while a
content conflict fails before missing scaffold files are written. Workspace
scaffolds contain a program and a library connected through
`[dependencies].workspace`. See [Initializing projects and workspaces](initializing.md).

## Building artifacts

```sh
fpas build my-app.fpasprj
fpas build suite.fpasworkspace
fpas build --executable my-app.fpasprj
fpas build --executable --name hello suite.fpasworkspace
fpas build
```

Program artifacts are named from `project.name` and written beside the
`.fpasprj`. Repeating an unchanged build reports reuse and leaves the compatible
artifact in place. A test project has multiple entry programs, so it validates
those programs but does not produce one shared `.fpascp`.

`--executable` requires a program project or a workspace containing exactly one
program. Project applications are written beside the `.fpasprj`; workspace
applications are written beside the `.fpasworkspace`. The resulting file
contains the host-native runner and validated `.fpascp`, and runs without
`fpas`, the separate runner, sources, manifests, `.fpascu`, `.fpascp`, or the
source standard library. The complete bundle is validated and written to a
same-directory staging file before one atomic replacement publishes it.
Repeating the command therefore keeps the previous application in place until
the replacement commits; a failed publication leaves that previous file
unchanged. On Linux, executable permissions are applied before publication.

`fpas build --std-lib <directory>` uses the same complete source standard
library override as the other compiler commands.

## Automatic compiled-unit builds

Project- and workspace-aware `check`, `run`, and `test` commands automatically
build and reuse source-adjacent `.fpascu` unit sidecars. Project- and
workspace-aware `run` also produces or reuses the program's `.fpascp`. Running
`fpas build` explicitly is not required before those commands. Successful
artifact reuse is silent.

The compiler validates content hashes, compiler and bytecode compatibility, compilation options,
and direct dependency interface hashes rather than relying on timestamps. Invalid derived files
are rebuilt from their `.fpas` source. Plain standalone `.fpas` programs and directory checks do
not create compiled-unit sidecars. A directory check may reuse compatible sidecars that already
exist, but newly compiled units remain in memory.

For project- and workspace-aware commands, a source standard-library override reads and rebuilds
matching sidecars beside the sources selected by `--std-lib`. Plain source and directory checks
keep newly compiled standard-library units in memory as well. Other commands need compatible
sidecars already present when the selected standard-library directory is read-only.

## Running programs

```sh
fpas run hello.fpas
fpas run my-app.fpasprj
fpas run suite.fpasworkspace
fpas run my-app.fpascp
fpas run my-app.fpasprj -- input.txt verbose
fpas run
```

`fpas run` does not accept a directory path. A project or workspace run keeps
sources authoritative and rebuilds a missing, stale, incompatible, or corrupt
`.fpascp` before execution. A directly passed `.fpascp` has no source input to
rebuild from, so invalid images fail with an actionable diagnostic. Validation
checks constant and jump operands, requires root control flow to reach `Halt`
or a root-level `Return` without falling into callable bodies, verifies
that name operands are strings, and resolves direct calls and closure targets
against the callable table (including direct-call arity) before the VM starts.

## Checking without running

Use `fpas check` to parse, link, and type-check without executing code:

```sh
fpas check my-lib.fpasprj
fpas check my-app.fpasprj
fpas check hello.fpas
fpas check suite.fpasworkspace
fpas check
```

With no path, `fpas check` discovers a single `.fpasworkspace` in the current directory first,
otherwise a single `.fpasprj`. Library projects type-check here the same as program projects. When
given a directory, all `.fpas` files under that tree form one source set: units are checked
together, and every program is checked against those sibling units. Directory discovery is
deterministic, skips `target` directories and symbolic links, and aborts with the affected path when a
directory entry cannot be read. `fpas fmt` and `fpas test` use the same traversal policy.

The parser shares a nesting budget of 128 levels across expressions, statements, type expressions,
and routine declarations. Source that exceeds the budget stops parsing with diagnostic `F1009`
instead of exhausting the compiler's native stack.

## Terminal diagnostics

Compiler and runtime diagnostics use the stable form
`path:line:column: severity[Fxxxx]: message`. When no source path is available, the location starts
with `line:column`. Printable Unicode and Windows path separators are preserved. Control characters
in paths and non-line-ending control characters in messages or help text are rendered as visible
escapes, so diagnostic content cannot inject terminal control sequences or synthetic location
lines.

`CRLF`, bare `CR`, and `LF` inside messages or help text are normalized as logical line breaks.
Every continuation is explicit: message continuations use `  message: ` and every help line uses
`  help: `. A source path always remains on one physical output line.

## Formatting project sources

`fpas fmt` accepts the same project and workspace manifests used by the other
project commands. It formats exactly the sources loaded from those manifests,
including a program's main file:

```sh
fpas fmt my-app.fpasprj
fpas fmt --check suite.fpasworkspace
fpas fmt --stdout src/main.fpas
```

Formatting never builds or runs the project. Parser diagnostics retain the
source path, stable code, and location used by editor Problems integration.
In-place formatting writes the complete result to a temporary file beside the
source and atomically replaces the source only after the write succeeds. A
write or commit failure leaves the original source unchanged, including its
existing file permissions.
`--stdout` accepts exactly one `.fpas` file, writes only the formatted source to
stdout, and leaves that file unchanged. It cannot be combined with `--check`;
use `--check --list` to list formatting drift instead.

## See also

- [Projects](projects.md)
- [Workspaces](workspaces.md)
- [Initializing projects and workspaces](initializing.md)
- [`Std.Test`](../std/testing/test.md)
