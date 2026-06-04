# 10. Projects

A project groups source files into a buildable unit. Projects are defined by a `.fpasprj` file using TOML format.

Multi-file programs are composed with project source lists plus `unit` / `uses`. There is no source-level include mechanism such as `{$I}` or `{$INCLUDE}`.

## CLI Usage

- `fpas` (no arguments) — searches the current directory for a `.fpasprj` file.
  - No match: error.
  - One match: loads that project file.
  - Multiple matches: error — pass the desired `.fpasprj` path explicitly.
- `fpas <path>` — detects input type by extension:
  - `.fpas` — runs as a single source file with a `program` declaration (no project needed).
  - `.fpasprj` — loads as a project file.
  - Other extensions — error.
- `fpas` with more than one argument — usage error.
- `fpas -h` / `fpas --help` — prints usage to stdout and exits successfully.
- `fpas -V` / `fpas --version` — prints the compiler version to stdout and exits successfully.

## Project File Format

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

### `[project]` Section

| Field | Required | Description |
|---|---|---|
| `name` | Yes | Project name. Any non-empty string. |
| `version` | No | Free-form version string. |
| `kind` | Yes | `"program"` or `"library"`. |
| `main` | Program only | Path to the program file (relative to project root or absolute). |

### Project Kinds

- **`program`** — produces an executable. Requires `main` pointing to a file with a `program` declaration. The entry point is exactly one main program file per project.
- **`library`** — a reusable library. Must not define `main`. Source files are expected to use `unit` declarations. Other projects consume libraries via `[dependencies].projects`. The CLI cannot execute a library project directly; run a `program` project that depends on it instead.

### `[dependencies]` Section

Declares other `.fpasprj` files whose library sources are merged into this project before linking.

| Field | Required | Description |
|---|---|---|
| `projects` | No | Array of paths to library `.fpasprj` files. Omitted or empty means no project dependencies. |

Each `projects` entry can be:

- **Relative path** — resolved relative to this project's root (the directory containing the `.fpasprj` file). Use this for monorepos, for example `"../libs/acme-utils/acme-utils.fpasprj"`.
- **Absolute path** — used as-is. Use this when the library lives anywhere on the filesystem outside the consumer tree.

Rules:

- Every dependency must be a `kind = "library"` project. Depending on a `program` project is an error.
- Dependencies are loaded **transitively**: if library B depends on library C, a program that depends only on B also receives C's sources.
- Cyclic `dependencies.projects` chains are rejected.
- Unit names must remain unique across the consumer and all transitive library sources (case-insensitive), same as within a single project.
- Library sources are linked only when reachable through `uses` from the program entry point (see [09-units.md](09-units.md)).

### `[sources]` Section

Lists all source files belonging to the project. Each source file declares its namespace via a `unit` declaration (see [09-units.md](09-units.md)).

| Field | Required | Description |
|---|---|---|
| `include` | Yes | Array of file paths or glob patterns. Must contain at least one entry. |

#### Include Patterns

These `include` entries belong to the project file format only. They are not related to Pascal compiler directives.

Each `include` entry can be:

- **Glob** — e.g. `"src/**/*.fpas"`, `"lib/*.fpas"`.
- **Relative path** — e.g. `"src/utils.fpas"`. Resolved relative to the project root.
- **Absolute path** — e.g. `"/home/user/shared/common.fpas"`.

Entries may be mixed freely. All matched files must have the `.fpas` extension.

> `exclude` patterns are not yet supported.

### Source File Rules

- The program file (`main`) is automatically excluded from the source list, even if matched by an include pattern.
- If another source file contains a `program` declaration instead of `unit`, a warning is emitted and the file is skipped.
- If an explicit path does not exist or an include pattern matches no files, the compiler emits an error.
- If multiple entries resolve to the same file, a warning is emitted and the duplicate is ignored.
- Duplicate unit names (case-insensitive) across different files are rejected.

## Example: Single Project

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

## Example: Program With a Library Dependency

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

## Workspaces (Planned)

A workspace groups multiple projects, similar to a Visual Studio solution. A workspace file would reference one or more `.fpasprj` files and allow cross-project builds and shared dependencies. This feature is not yet implemented.
