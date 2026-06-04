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

The program project lists the library in `[dependencies].projects`. Paths are relative to each `.fpasprj` file (or absolute for libraries outside the tree).
