# CLI

The `fpas` command-line interface discovers projects, type-checks, runs programs, and executes test bundles.

## Usage

- `fpas` (no arguments) — prints usage to stdout and exits successfully.
- `fpas run` — discovers what to run in the current directory:
  - If a `.fpasworkspace` file exists: runs the sole `kind = "program"` member; errors when there are zero or multiple program members.
  - Otherwise searches for a `.fpasprj` file (no match, one match, or multiple matches with the same rules as before).
- `fpas run <path>` — detects input type by extension:
  - `.fpas` — runs as a single source file with a `program` declaration (no project needed).
  - `.fpasprj` — loads as a project file.
  - Other extensions — error.
- `fpas run` with more than one positional path argument — usage error.
- `fpas check [<path>]` — type-check a `.fpas`, directory of `.fpas` files, `.fpasprj`, or `.fpasworkspace` without running. With no path, discovers `.fpasworkspace` or `.fpasprj` in the current directory.
- `fpas test [<path>]` — run `*_test.fpas` programs and print a pass/fail/skip summary. With no path, discovers a workspace or `.fpasprj` like `fpas check`. Flags: `--list`, `--fail-fast`, `--strict` (exit `1` when any test called `Skip`), `--filter <pattern>`, `--report json`, `--timeout <secs>`, `--jobs <n>` (`0` = available CPU parallelism), `--script <path>`. Sidecars beside each test file (all optional): `<test>.script.toml` (project overrides), `<test>.expect.stdout`, `<test>.expect.screen` (TUI), `<test>.expect.pixels` (headless graph). See [`Std.Test`](../std/testing/test.md). `--list` and `--report json` write results to stdout; progress lines stay on stderr.
- `fpas -h` / `fpas --help` — prints usage to stdout and exits successfully.
- `fpas -V` / `fpas --version` — prints the compiler version to stdout and exits successfully.

Program arguments after `--` require `fpas run` and are visible through `Std.Args` when running programs.

## Running programs

```sh
fpas run hello.fpas
fpas run my-app.fpasprj
fpas run my-app.fpasprj -- input.txt verbose
fpas run
```

`fpas run` does not accept a directory path; pass a `.fpas` program file or a `.fpasprj` project.

## Checking without running

Use `fpas check` to parse, link, and type-check without executing code:

```sh
fpas check my-lib.fpasprj
fpas check my-app.fpasprj
fpas check hello.fpas
fpas check suite.fpasworkspace
fpas check
```

With no path, `fpas check` discovers a single `.fpasworkspace` in the current directory first, otherwise a single `.fpasprj`. Library projects type-check here the same as program projects. When given a directory, every `.fpas` file under that tree is type-checked as a standalone source file (the same rules as `fpas fmt` for discovery).

## See also

- [Projects](projects.md)
- [Workspaces](workspaces.md)
- [`Std.Test`](../std/testing/test.md)
