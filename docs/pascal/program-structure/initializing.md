# Initializing projects and workspaces

`fpas init` creates formatted, immediately checkable Functional Pascal scaffolds. It is
non-interactive: every input can be passed on the command line, making the command suitable for
terminals, scripts, and coding agents.

## Program project

```sh
fpas init project hello
fpas check hello/hello.fpasprj
fpas run hello/hello.fpasprj
```

The command creates:

```text
hello/
  .gitignore
  hello.fpasprj
  src/
    main.fpas
```

The manifest uses `kind = "program"`, and `src/main.fpas` contains a formatted program that writes
a greeting through `Std.Console`.

## Library project

```sh
fpas init library greet --unit Demo.Greet
fpas check greet/greet.fpasprj
```

The optional `--unit` value selects the exported, qualified unit name. Without it, `fpas init`
derives a Pascal-case unit name from the project name. For example, `my-library` becomes
`MyLibrary`.

The command creates a `kind = "library"` manifest with `[exports].units` and one public `Message`
function in `src/`.

## Workspace

```sh
fpas init workspace acme-suite
fpas check acme-suite/acme-suite.fpasworkspace
fpas run acme-suite/acme-suite.fpasworkspace
```

A workspace scaffold is complete rather than empty: it contains one program and one library. The
program consumes the library through `[dependencies].workspace`.

```text
acme-suite/
  .gitignore
  acme-suite.fpasworkspace
  apps/
    acme-suite/
      acme-suite-app.fpasprj
      src/main.fpas
  libs/
    acme-suite-core/
      acme-suite-core.fpasprj
      src/core.fpas
```

## Target directory

By default, the target directory is `<name>` below the current directory. Use `--path` to select a
different location:

```sh
fpas init project portal --path apps/portal
fpas init library greet --path libs/greet --unit Demo.Greet
fpas init workspace acme-suite --path work/acme-suite
```

Names use ASCII letters and digits with optional single `-` or `_` separators and must start with a
letter. This restriction lets `fpas init` derive valid source identifiers and portable manifest
paths deterministically. A name that would produce a reserved Functional Pascal keyword is rejected
with an example of a more specific name.

## Preview and machine-readable output

`--dry-run` prints the complete plan without creating directories or files. `--report json`
selects a machine-readable stdout report:

```sh
fpas init workspace acme-suite --dry-run --report json
```

The report contains `status`, `kind`, `name`, `root`, `manifest`, and `files`. Human-readable output
uses the same field names. Errors are written to stderr and return a nonzero exit code.

## Existing files and retries

`fpas init` never overwrites an existing file. If every planned file already has the generated
content, repeating the command succeeds with `status: unchanged`. If an existing planned file has
different content or is not a regular UTF-8 file, the command fails before writing any missing
scaffold files and lists the conflicts.

Derived `.fpascp` and `.fpascu` files and their persistent lock files are excluded by the generated
`.gitignore`. `fpas init` does not initialize a Git repository, build code, or run a program.

## Command help

Use layered help to inspect only the relevant scaffold:

```sh
fpas init --help
fpas init project --help
fpas init library --help
fpas init workspace --help
```

## See also

- [Projects](projects.md)
- [Workspaces](workspaces.md)
- [CLI](cli.md)
