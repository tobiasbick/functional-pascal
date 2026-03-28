# 10. Projects

A project groups source files into a buildable unit. Projects are defined by a `.fpasprj` file using TOML format.

## CLI Usage

- `fpas` (no arguments) — searches the current directory for a `.fpasprj` file.
  - No match: error.
  - One match: loads that project file.
  - Multiple matches: error — pass the desired `.fpasprj` path explicitly.
- `fpas <path>` — detects input type by extension:
  - `.fpas` — runs as a single source file (no project needed).
  - `.fpasprj` — loads as a project file.
  - Other extensions — error.
- `fpas` with more than one argument — usage error.

## Project File Format

```toml
[project]
name = "my-app"
version = "1.0.0"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]
```

### `[project]` Section

| Field | Required | Description |
|---|---|---|
| `name` | Yes | Project name. Any non-empty string. |
| `version` | No | Free-form version string. |
| `kind` | Yes | `"program"` or `"library"`. |
| `main` | Program only | Path to the program file (relative to project root or absolute). |

### Project Kinds

- **`program`** — produces an executable. Requires `main` pointing to a file with a `program` declaration. There is exactly one program file per project.
- **`library`** — a reusable library. All files use `unit` declarations. Must not define `main`.

### `[sources]` Section

Lists all source files belonging to the project. Each source file declares its namespace via a `unit` declaration (see [09-units.md](09-units.md)).

| Field | Required | Description |
|---|---|---|
| `include` | Yes | Array of file paths or glob patterns. Must contain at least one entry. |

#### Include Patterns

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

## Example Project

Directory structure:

```
my-app/
  my-app.fpasprj
  src/
    main.fpas
    math.fpas
    color.fpas
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

## Workspaces (Planned)

A workspace groups multiple projects, similar to a Visual Studio solution. A workspace file would reference one or more `.fpasprj` files and allow cross-project builds and shared dependencies. This feature is not yet implemented.
