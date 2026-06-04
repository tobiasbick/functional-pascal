# Monorepo example (library + program + workspace)

Layout:

```text
monorepo/
  monorepo.fpasworkspace
  libs/greet/          kind = library
  apps/hello/          kind = program, depends on greet
```

## Run

From the repository root:

```sh
fpas examples/pascal/monorepo/apps/hello/hello.fpasprj
```

From the workspace directory (runs the sole program member `hello`):

```sh
cd examples/pascal/monorepo
fpas
```

## Check

Type-check the whole workspace (library + program):

```sh
cd examples/pascal/monorepo
fpas check
```

Or check individual projects:

```sh
fpas check libs/greet/greet.fpasprj
fpas check apps/hello/hello.fpasprj
```

The program project lists the library as `workspace = ["greet"]`, matching `project.name` in `greet.fpasprj`. The library exposes only `Demo.Greet` via `[exports].units`; `Demo.Greet.Internal` is private to the library project.

Alternatively use `[dependencies].projects` with a relative or absolute path to the library manifest.
