# Monorepo example (library + program + workspace)

Layout:

```text
monorepo/
  monorepo.fpasworkspace
  libs/greet/          kind = library
  apps/hello/          kind = program, depends on greet
  apps/tests/          kind = test, depends on greet
```

## Run

From the repository root:

```sh
fpas run examples/pascal/monorepo/apps/hello/hello.fpasprj
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

## Test

Run the workspace test member (`apps/tests`) from the workspace root:

```sh
cd examples/pascal/monorepo
fpas test
```

Or filter to one test file:

```sh
fpas test --filter greet_test
```

The test project uses `workspace = ["greet"]` like the hello app and asserts on `Demo.Greet.Message()`.
