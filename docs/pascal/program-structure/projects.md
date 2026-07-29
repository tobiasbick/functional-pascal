# Projects

A project groups source files into a buildable unit. Projects are defined by a `.fpasprj` file using TOML format.

Multi-file programs compose source lists in the project file with `unit` declarations and `uses` imports.

## Project file format

```toml
[project]
name = "my-app"
version = "1.0.0"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]

[dependencies]
projects = ["../my-lib/my-lib.fpasprj"]
```

### `[project]` section

| Field | Required | Description |
|---|---|---|
| `name` | Yes | Project name. Any non-empty string; names used for program artifacts must not contain path separators. |
| `version` | No | Free-form version string. |
| `kind` | Yes | `"program"`, `"library"`, or `"test"`. |
| `main` | Program only | Path to the program file (relative to project root or absolute). |

### Project kinds

- **`program`** — produces a linked `.fpascp` bytecode image through `fpas build`.
  Requires `main` pointing to a file with a `program` declaration. The entry
  point is exactly one main program file per project.
- **`library`** — reusable code as `unit` files. Other projects consume libraries via `[dependencies].projects`. Run library code through a `program` project that depends on it (`fpas check` type-checks a library manifest directly).
- **`test`** — a test bundle for `fpas test`. Lists `unit` helpers and `*_test.fpas` program entry files in `[sources]`. Optional `[test.overrides."<file>_test.fpas"]` tables set per-test `script` and `headless_graph` for the runner. In a workspace, `fpas test` with no path runs tests from all `kind = "test"` members only.

Library dependencies remain manifest- and source-based: projects identify libraries by
`.fpasprj` path or workspace member name. Their units are compiled independently into
source-adjacent `.fpascu` sidecars and linked through their public interfaces; dependency
declarations are not merged into the consumer's program AST.

### `[dependencies]` section

Declares other `.fpasprj` files whose units are available to this project's unit graph.

| Field | Required | Description |
|---|---|---|
| `projects` | No | Array of paths to library `.fpasprj` files. Omitted or empty means no path-based dependencies. |
| `workspace` | No | Array of `project.name` values from members of an enclosing `.fpasworkspace` file. Resolved by walking upward from the consumer project. |

Each `projects` entry can be:

- **Relative path** — resolved relative to this project's root (the directory containing the `.fpasprj` file). Use this for monorepos, for example `"../libs/acme-utils/acme-utils.fpasprj"`.
- **Absolute path** — used as-is. Use this when the library lives anywhere on the filesystem outside the consumer tree.

Rules:

- Every dependency must be a `kind = "library"` project.
- `workspace` entries require a `.fpasworkspace` ancestor (in the consumer's parent directories). Names match `project.name` in member manifests (case-insensitive).
- Dependencies are loaded **transitively**: if library B depends on library C, a program that depends only on B also receives C's sources.
- Cyclic `dependencies.projects` chains are rejected.
- Unit names must remain unique across the consumer and all transitive library sources (case-insensitive), same as within a single project.
- Library sources are linked only when reachable through `uses` from the program entry point (see [Units](units.md)).
- Missing, stale, corrupt, or compiler/bytecode-incompatible `.fpascu` files are rebuilt
  automatically beside their `.fpas` sources.
- `.fpascu` is a derived build artifact, not a dependency syntax and not a source-less package
  format. The `.fpasprj` manifest and `.fpas` sources remain required and authoritative.

### Build outputs

`fpas build <project.fpasprj>` uses `project.name` for program artifacts:

```text
[project]
name = "hello"
kind = "program"
```

produces or reuses `hello.fpascp` beside the project manifest. Library projects
build their source-adjacent `.fpascu` files and do not produce `.fpascp`.
Test projects build helper-unit sidecars and validate all test programs without
creating one shared program image. Both artifact types are derived outputs and
are ignored by Git.

`fpas build --executable <project.fpasprj>` additionally produces a
host-native single-file application beside the project manifest. Its default
base name is `project.name`; `--name <name>` overrides only the application and
output name. Windows produces `<name>.exe`, while Linux produces executable
`<name>`.

### Exports section (library projects only)

Optional on `kind = "library"` projects. Controls which units other projects may import across a dependency boundary.

| Field | Required | Description |
|---|---|---|
| `units` | Yes (when section present) | Array of unit names (`unit` declarations) that dependents may reference in `uses`. |

Rules:

- Omitted `[exports]` means **all units** in the library are importable by dependents (same as before).
- `[exports].units` lists unit names that dependents may reference in `uses`. Other units in the library remain available only inside the library via `uses`.
- Each name must match a `unit` in the library's `[sources]` (case-insensitive).
- `[exports]` applies to `kind = "library"` projects only.
- Declarations without `public` remain internal to their unit; `[exports]` hides whole units from other projects.

Example:

```toml
[exports]
units = ["MyLib.Core"]
```

### `[sources]` section

Lists all source files belonging to the project. Each source file declares its namespace via a `unit` declaration (see [Units](units.md)).

| Field | Required | Description |
|---|---|---|
| `include` | Yes | Array of file paths or glob patterns. Must contain at least one entry. |
| `exclude` | No | Array of file paths or glob patterns. Removes matches from the include set after inclusion. |

#### Include and exclude patterns

Each `include` entry can be:

- **Glob** — e.g. `"src/**/*.fpas"`, `"lib/*.fpas"`.
- **Relative path** — e.g. `"src/utils.fpas"`. Resolved relative to the project root.
- **Absolute path** — e.g. `"/home/user/shared/common.fpas"`.

Entries may be mixed freely. All **included** files must have the `.fpas` extension.

`exclude` uses the same path and glob rules. Exclude globs may match zero files without error. Non-`.fpas` paths in exclude are ignored when they do not appear in the include set.

### Source file rules

- The program file (`main`) is automatically excluded from the source list, even if matched by an include pattern.
- If another source file contains a `program` declaration instead of `unit`, a warning is emitted and the file is skipped.
- If an explicit path does not exist or an include pattern matches no files, the compiler emits an error.
- If multiple entries resolve to the same file, a warning is emitted and the duplicate is ignored.
- Duplicate unit names (case-insensitive) across different files are rejected.

## Example: single project

Directory structure:

```
my-app/
  my-app.fpasprj
  src/
    main.fpas
    math.fpas
```

`my-app.fpasprj`:

```toml
[project]
name = "my-app"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]
```

`src/main.fpas`:

```pascal
program MyApp;
uses MyApp.Math, Std.Console;
begin
  WriteLn(Add(3, 4));
end.
```

`src/math.fpas`:

```pascal
unit MyApp.Math;

function Add(A: integer; B: integer): integer;
begin
  return A + B
end;
```

## Example: program with a library dependency

The `suite/`, `libs/acme-utils/`, and `apps/portal/` paths below are **illustrative** — they document the project-file shape only and are **not** present in this repository. Runnable samples live under [`examples/pascal/library-deps/`](../../../examples/pascal/library-deps/) (path-based `[dependencies].projects`) and [`examples/pascal/monorepo/`](../../../examples/pascal/monorepo/) (workspace + library).

Monorepo layout:

```
suite/
  libs/
    acme-utils/
      acme-utils.fpasprj
      src/math.fpas
  apps/
    portal/
      portal.fpasprj
      src/main.fpas
```

`libs/acme-utils/acme-utils.fpasprj`:

```toml
[project]
name = "acme-utils"
kind = "library"

[sources]
include = ["src/**/*.fpas"]
```

`libs/acme-utils/src/math.fpas`:

```pascal
unit Acme.Math;

function Add(A: integer; B: integer): integer;
begin
  return A + B
end;
```

`apps/portal/portal.fpasprj`:

```toml
[project]
name = "portal"
kind = "program"
main = "src/main.fpas"

[dependencies]
projects = ["../../libs/acme-utils/acme-utils.fpasprj"]
# or, inside a workspace tree:
# workspace = ["acme-utils"]

[sources]
include = ["src/**/*.fpas"]
```

`apps/portal/src/main.fpas`:

```pascal
program Portal;
uses Acme.Math, Std.Console;
begin
  WriteLn(Add(3, 4));
end.
```

A library outside the monorepo uses the same `[dependencies].projects` field with an absolute path to its `.fpasprj` file.

## See also

- [Units](units.md)
- [CLI](cli.md)
- [Workspaces](workspaces.md)
