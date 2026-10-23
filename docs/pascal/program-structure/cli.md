# CLI

The `fpas` command-line interface creates scaffolds, builds artifacts, discovers
projects, type-checks, runs programs, executes test bundles, and exposes the
installed toolchain to editor clients.

Standalone execution and bundled executables authorize the explicit process-lifecycle operations
in [`Std.Server`](../std/network/server.md). Programs must still opt into signal observation and
forced process termination. Embedded VMs deny these operations unless their host authorizes them;
the source debugger and in-process test hosts do not grant this authority automatically.

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
- `fpas doc <name> [<path>]` — print the signature, documentation and handbook page of a
  declaration or built-in operation without compiling; see
  [Looking up declarations](#looking-up-declarations).
- `fpas query refs|definition|type <file.fpas:line:column>` — find references, definitions,
  or checked expression types as JSON through the shared language service, without
  writing artifacts. `--context <manifest>` selects the boundary; `--timeout`
  bounds the complete request and `refs --limit` bounds retained uses. Incomplete
  coverage exits 3. See [Source queries](../tools/source-queries.md).
- `fpas query rename <file.fpas:line:column> <new-name>` — safely rename a
  declaration and its references, returning the edits as JSON. `--dry-run`
  validates and lists edits without writing. `--project <path>` selects the
  project, workspace, or folder context. See [Safe rename](../tools/source-queries.md#safe-rename).
- `--diagnostics <text | json>` — accepted by `check`, `build`, `run`, and `test`; selects the stderr diagnostic format (see [Machine-readable diagnostics](#machine-readable-diagnostics)).
- `fpas test [<path>]` — run `*_test.fpas` programs and print a pass/fail/skip summary. With no path, discovers a workspace or `.fpasprj` like `fpas check`. Flags: `--list`, `--fail-fast`, `--strict` (exit `1` when any test called `Skip`), `--show-output` (print captured standard output of passing tests too), `--filter <pattern>`, repeatable `--file <path>` (select an exact discovered file, relative to the current directory), `--report json`, `--timeout <secs>` (default: `300`), `--jobs <n>` (`0` = available CPU parallelism), `--script <path>`. Sidecars beside each test file (all optional): `<test>.script.toml` (scripted input), `<test>.expect.stdout`, and `<test>.expect.screen` (TUI). See [`Std.Test`](../std/testing/test.md). `--list` and `--report json` write results to stdout; JSON test entries identify their full source paths, including nested directories. Progress lines stay on stderr. A failing test's captured standard output is always printed below its result line; `--show-output` does the same for passing tests. A write failure on contracted stdout or summary output returns nonzero instead of reporting success. Each test's timeout budget starts before its isolated worker is spawned and covers worker preparation and VM execution. Tests run in a terminable process tree, so blocking startup, VM, or host calls cannot extend the timeout indefinitely.
- `fpas debug [<path>]` — run once to the first breakpoint, runtime error or
  completion and return one bounded JSON snapshot on stdout. Repeat `--break
  path:line[:column]` and `--print expression` for targeted inspection; `--script`
  supplies runner-compatible input. The whole-invocation deadline defaults to
  30 seconds. Repeat `--trace selector` to retain scalar global/record-field or
  exact `Routine::Name` local/parameter/capture changes through stored fields or
  literal array positions, with `--trace-limit count` for bounded event capacity; see
  [Tracepoints](../tools/debugger-tracepoints.md). Use `--schedule-seed N` for
  [controlled task ordering](../tools/debugger-scheduling.md) with virtual time
  and text-line input. See
  [Batch debugging](../tools/debugger-batch.md) for the result
  schema, bounds and exit codes. Explicit `--protocol jsonl|dap` selects the
  existing long-lived source-debugger sessions; see [Source debugger](../tools/debugger.md).
- `fpas env --json` — print the installed toolchain contract. Schema version 1
  reports the FPAS version, absolute executable path, and adjacent source
  standard-library directory as JSON.
- `fpas lsp` — serve the Functional Pascal Language Server Protocol over
  standard input and output. Editor clients use this entry point so language
  features come from the same toolchain as builds, runs, and debugging.
- `fpas fmt [<path> ...]` — format source files, directories, projects, or
  workspaces. A program project includes its `project.main` source as well as
  unit sources. `--check` changes no files; when formatting would change any,
  it names them on stderr with a hint and exits with status 2. `--list` with
  `--check` prints only those paths on stdout instead. `--stdout <file.fpas>` formats
  one source file to stdout without modifying it and cannot be combined with
  `--check`.
- `fpas -h` / `fpas --help` — prints the short command overview to stdout and exits successfully.
- `fpas init --help`, `fpas init <kind> --help`, `fpas build --help`,
  `fpas run --help`, `fpas check --help`, `fpas doc --help`, `fpas query --help`,
  `fpas test --help`, `fpas debug --help`, `fpas fmt --help`,
  `fpas env --help`, and `fpas lsp --help` — print focused command help with
  valid examples and exit successfully.
- `fpas -V` / `fpas --version` — prints the compiler version to stdout and exits successfully.
- `fpas build --std-lib <directory> …`, `fpas run --std-lib <directory> …`,
  `fpas check --std-lib <directory> …`, `fpas doc --std-lib <directory> …`,
  `fpas query refs|definition|type|rename --std-lib <directory> …`, and
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
fpas env --help
fpas lsp --help
```

## Installed toolchain discovery

Editor integrations should resolve one `fpas` executable and query it with:

```sh
fpas env --json
```

The command is non-interactive and writes only the versioned JSON report to
stdout. The reported standard-library directory belongs to that executable;
clients should not independently combine a compiler, language server, and
library from different installations. Start the matching language server with
`fpas lsp`.

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
Program builds also print `Debug source root: ...`: the canonical common root
used for program and resolved-dependency source paths. Pass it to
`fpas debug image.fpascp --source-root <dir>`; see
[source roots for debugging](compiled-programs.md#source-roots-for-debugging).

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
checks register operands, constants, layout references, and jump targets. Function bodies
occupy disjoint instruction ranges; branches and fallthrough cannot escape those ranges.
The entry function takes no parameters or captures and uses the `unit` return convention;
normal completion uses `Return`. Direct calls and closure targets are checked against the
function table, including direct-call arity, before the VM starts.

To retain runtime failure data, use `fpas run <path> --failure-dump <new-file>`.
The option also works with discovery and direct `.fpascp` execution. It writes
one bounded JSON document only when a runtime diagnostic escapes the run; success
and pre-execution failures create no report or report directories. Existing or
racing destinations are preserved. Publication failure adds FP4139 after the
original diagnostic, retaining runtime exit code `2`. Only `run` accepts this
option; flags after `--` remain program arguments. See the
[failure-report schema and limits](../tools/failure-dump.md).

To find where execution time goes, use `fpas run <path> --profile <new-file>`.
It writes one JSON report after the run, also when the run fails, with the same
new-file rule. Only `run` accepts this option. See [profile reports](../tools/profiler.md).

## Collection scheduling

`fpas run`, `fpas test` and `fpas debug` use automatic local cycle collection.
These positive decimal integer options override the per-domain minimum thresholds:

| Option | Default | Meaning |
| --- | --- | --- |
| `--gc-cell-threshold=N` | 4096 | Newly allocated cells before a collection request |
| `--gc-mutation-threshold=N` | 8192 | Successful cell mutations before a running-task request |
| `--gc-poll-interval=N` | 256 | Completed execution batches before a periodic request |

Each threshold grows with the graph the last pass traced: allocations and
mutations by one per traced shared value, batches by two. A large live graph is
therefore traced in proportion to the work between passes; unreachable cycles can
then grow to about the size of the live graph before they are collected.
The first reached threshold requests collection at the next safe point. A batch is
an execution dispatch unit, not a millisecond or individual instruction. Busy passes
keep pending work. Root releases and quiescent mutations are serviced without
waiting for these thresholds. The options never bypass root publication or domain
exclusion, and do not promise a maximum collection pause. Reference counting still
releases acyclic values immediately.

Both `--option=N` and `--option N` forms are accepted. Omitted values use defaults;
zero, negative, malformed, overflowing or duplicate values are argument errors.
Source/project/workspace and compiled-image runs, isolated test bodies and hooks,
and batch/JSONL/DAP debugger execution use the selected settings. Child tasks,
callback sessions and debugger helper domains inherit them. Options after `--`
remain program arguments. Check/build/format commands do not accept these options.

```sh
fpas run app.fpasprj --gc-cell-threshold=2048 --gc-poll-interval=128
fpas test tests/suite.fpasprj --gc-mutation-threshold=4096
fpas debug app.fpasprj --protocol jsonl --gc-cell-threshold=2048
```

These are heuristic starting values. Controlled local measurements may guide later
tuning; cloud correctness runs are not performance baselines. See
[runtime ownership](runtime-ownership.md#cycle-reclamation).

## Checking without running

Use `fpas check` to parse, link, and type-check without executing code:

```sh
fpas check my-lib.fpasprj
fpas check my-app.fpasprj
fpas check hello.fpas
fpas check suite.fpasworkspace
fpas check
```

`fpas check --fix` first applies the machine-applicable [fixes](../tools/diagnostics.md#fixes) of
the reported diagnostics in place, checks again, and repeats until a pass applies none, at most
ten passes. Then it reports the remaining diagnostics as usual and, in text mode, one summary
line such as `applied 7 fixes in 2 files`. A file or directory selection fixes its selected `.fpas`
sources. A project selection fixes its own `main` and files selected by its source include/exclude
rules, even when those files lie outside the project directory. A workspace selection includes
each selected member's own sources, including members outside the workspace directory.
Dependency-only projects and the standard library remain unchanged, even when a dependency's
directory is nested beneath the selected project directory. Files are replaced atomically.
`--fix` does not format; run `fpas fmt` afterwards. The exit code is
that of the final check. To see the fixes without writing, use `--diagnostics json`.

```sh
fpas check --fix my-app.fpasprj
```

With no path, `fpas check` discovers a single `.fpasworkspace` in the current directory first,
otherwise a single `.fpasprj`. Library projects type-check here the same as program projects. When
given a directory, all `.fpas` files under that tree form one source set: units are checked
together, and every program is checked against those sibling units. Directory discovery is
deterministic, skips `target` directories and symbolic links, and aborts with the affected path when a
directory entry cannot be read. `fpas fmt` and `fpas test` use the same traversal policy.

The parser shares a nesting budget of 128 levels across expressions, statements, type expressions,
and routine declarations. Source that exceeds the budget stops parsing with diagnostic `FP2009`
instead of exhausting the compiler's native stack.

## Looking up declarations

`fpas doc <name>` prints what the editor shows on hover for a name: its signature, its `//`
[documentation comment](../language/basics/comments.md) and, when known, the owning handbook
page. It covers intrinsic and source `Std` units, the operations on built-in types and the
declarations of the selected project. Nothing is compiled or written, and a project whose
sources do not check is still searched.

```sh
fpas doc Std.Fs.ReadText
fpas doc string.TrimLeft
fpas doc Std.Path
fpas doc TPoint.Distance my-app.fpasprj
fpas doc --report json array.Push
```

Names are case-insensitive. A name matches a declaration when it equals the qualified name or
ends with it at a dot, so `ReadText`, `Fs.ReadText` and `Std.Fs.ReadText` find the same routine;
every match is printed, so a short name can list several. Operations on built-in types are
written on the type: `string`, `array`, `dict`, `Option` or `Result`, for example `dict.Keys`.
A bare operation name such as `Length` lists it for every type. A unit name prints the unit's
comment and lists its declarations with the first line of each comment: all declarations for a
project unit, and public declarations for a `Std` unit.

`<path>` selects the project like `fpas check`; without it the current directory is used.
Without a project, only `Std` units and built-in operations are searched. A project's own
declarations are found whether public or not; `Std` units contribute public declarations only.
A unit's handbook page is the `docs/pascal/….md` path named in the comment before its `unit`
header.

`--report json` writes one JSON document to stdout:

```json
{"query":"string.TrimLeft","matches":[{"name":"string.TrimLeft","kind":"operation",
  "signature":"string.TrimLeft(): string","documentation":"Remove leading whitespace",
  "handbook":"docs/pascal/language/types/string/README.md","location":null,"members":[]}],
  "suggestions":[]}
```

| Field | Content |
| --- | --- |
| `kind` | `operation`, `factory`, `unit`, `function`, `procedure`, `method`, `type`, `enum`, `constant`, `variable`, `field` or `enum member` |
| `documentation` | The declaration comment as Markdown, or `null` |
| `handbook` | Repository-relative handbook page, or `null` |
| `location` | `path`, one-based `line` and Unicode-scalar `column` of the declared name; `null` for built-in operations. The path is relative to the current directory when it lies below it. |
| `members` | For a unit, its declarations with `name`, `signature` and `summary`: all project declarations, or public declarations for `Std` units |
| `suggestions` | Up to five closest names when nothing matches |

When nothing matches, `fpas doc` exits with status 1 and reports `FP4143` on stderr with the
closest names in its help; with `--report json` it still writes the document, with an empty
`matches` array.

## Terminal diagnostics

Compiler and runtime diagnostics use the stable form
`path:line:column: severity[FPnxxx]: message`. When no source path is available, the location starts
with `line:column`. Printable Unicode and Windows path separators are preserved. Control characters
in paths and non-line-ending control characters in messages or help text are rendered as visible
escapes, so diagnostic content cannot inject terminal control sequences or synthetic location
lines.

With matching source text, a numbered source line and caret marker appear before
the help. Long lines are cropped to 100 display cells; tabs use four-cell stops
and markers account for Unicode display width. Runtime point locations use one
caret. Text is colored automatically only when the final stderr is a terminal;
any nonempty `NO_COLOR` value disables it. Test workers inherit this policy.
Missing or unverified source text keeps the compact header/help. See
[diagnostic rendering](../tools/diagnostics.md#text-output) for range, cropping
and compiled-image source rules. JSON output keeps its existing schema.

`CRLF`, bare `CR`, and `LF` inside messages or help text are normalized as logical line breaks.
Every continuation is explicit: message continuations use `  message: ` and every help line uses
`  help: `. A source path always remains on one physical output line.

Every diagnostic has a stable `FPnxxx` code, including project, build, linker, argument and test
runner failures; positionless records omit the `line:column` part, for example
`error[FP4137]: Unknown option `--bogus`.`.

### Machine-readable diagnostics

`fpas check`, `fpas build`, `fpas run`, and `fpas test` accept `--diagnostics <text | json>`
(default `text`). With `--diagnostics json`, every stderr line is one UTF-8 JSON record:
a diagnostic as described in [shared diagnostics](../tools/diagnostics.md#rust-json-rendering-api)
or a program-output event as described below. Progress lines, result banners, test summaries, and captured test output are
omitted. Warnings use the same records with `"severity":"warning"`. Standard output is unchanged:
program output, build status lines, `--list`, and `--report json` still go to stdout. Exit codes
are the same as in text mode (`2` for a runtime error in `fpas run`). Argument errors also use JSON
when the arguments before `--` contain `--diagnostics json`.

```text
fpas check --diagnostics json src/main.fpas
{"kind":"diagnostic","code":"FP3003","severity":"error","phase":"sema","source":"/work/app/src/main.fpas","location":{"source_id":0,"start":{"line":3,"column":3},"end":{"line":3,"column":16}},"message":"Unknown procedure `MissingCall`","expected":null,"found":null,"hint":"Declare the function or procedure before use, or check the spelling.","fixes":[]}
```

`source` is the resolved path of the source file for parse, compile and project records. Runtime records of a program project or `.fpascp` name the portable source path stored in the compiled image, relative to the project directory, as in text mode. Runtime records have a known start and a `null` end.

In `fpas run` and `fpas test` with `--diagnostics json`, standard error of child processes started with
`Std.Proc.Run` is not inherited. Each of its lines becomes a program-output record, so program text
cannot be mistaken for a diagnostic:

```text
{"kind":"program-output","stream":"stderr","text":"child err"}
```

`text` is one line without its line ending. Records keep the order in which they are written.
Test workers buffer these records and forward them before the test's runtime diagnostic,
including when `--jobs` runs tests in parallel. Program-stderr records have a separate
8 MiB capture budget. At its limit the worker keeps complete records and emits one
program-output event with `text` equal to
`Program stderr was truncated at the 8 MiB capture limit.` Further program-stderr
records are omitted. Truncation does not change the test outcome, exit code or
stdout report, and does not consume the diagnostic output budget.
`Std.Proc.RunCapture` keeps its normal captured result.

A native application built with `fpas build --executable` keeps every command-line argument for the
program. Set the environment variable `FPAS_DIAGNOSTICS=json` to select the same JSON stderr stream
for its runtime records, startup failures and program-output records; any other value or no value
keeps text output.

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
