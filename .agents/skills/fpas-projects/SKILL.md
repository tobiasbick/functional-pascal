---
name: fpas-projects
description: >
  Guides Functional Pascal project manifests, workspaces, CLI workflows, and test bundles. Use when
  creating or editing `.fpasprj`, `.fpasworkspace`, dependencies, exports, `suite.fpasprj`, or running
  `fpas init`, `fpas build`, `fpas run`, `fpas check`, `fpas test`, `fpas fmt`. Also use when the user asks about project discovery,
  library linking, workspace members, or how to run/check tests.
---

# FPAS projects and tooling

Project-local guide for manifests and the `fpas` CLI. Spec details: [`docs/pascal/program-structure/`](../../../docs/pascal/program-structure/).

## Required reads

1. [`docs/pascal/program-structure/projects.md`](../../../docs/pascal/program-structure/projects.md)
2. [`docs/pascal/program-structure/workspaces.md`](../../../docs/pascal/program-structure/workspaces.md)
3. [`docs/pascal/program-structure/cli.md`](../../../docs/pascal/program-structure/cli.md)
4. [`.agents/skills/fpas-authoring/SKILL.md`](../fpas-authoring/SKILL.md) — writing `.fpas` sources referenced by manifests

After behavior changes: [`.agents/skills/fpas-change-checklist/SKILL.md`](../fpas-change-checklist/SKILL.md).

Workflow calibration: [references/examples.md](references/examples.md).

## Project kinds

| `kind` | `main` | Purpose |
|--------|--------|---------|
| `program` | Required — path to `program` file | Runnable app |
| `library` | Omit | Units consumed via `[dependencies]` |
| `test` | Omit | Bundle for `fpas test` (`*_test.fpas` programs) |

Rules:

- Dependencies name source projects; imported units are built independently into derived, source-adjacent `.fpascu` sidecars.
- Every `dependencies.projects` entry must be `kind = "library"`.
- `workspace` dependency names match `project.name` in an enclosing `.fpasworkspace`.
- Library `[exports].units` limits which units dependents may `uses` (omit section = all units exportable).
- `fpas` validates, reuses, or rebuilds compatible sidecars automatically. Do not hand-edit or commit them.
- There is no `.fpaslib` container, package registry, lockfile, or global artifact cache.

## Minimal manifests

### Program project

```toml
[project]
name = "hello"
kind = "program"
main = "src/main.fpas"

[dependencies]
workspace = ["greet"]

[sources]
include = ["src/**/*.fpas"]
```

### Library project

```toml
[project]
name = "greet"
kind = "library"

[exports]
units = ["Demo.Greet"]

[sources]
include = ["src/**/*.fpas"]
```

### Test bundle

```toml
[project]
name = "fpas-regression-tests"
kind = "test"

[sources]
include = [
  "stdlib/**/*_test.fpas",
  "concurrency/**/*_test.fpas",
  "runner/**/*_test.fpas",
]
```

Canonical copies: [`tests/suite.fpasprj`](../../../tests/suite.fpasprj), [`examples/pascal/monorepo/`](../../../examples/pascal/monorepo/).

### Workspace

```toml
[workspace]
name = "fpas-monorepo-example"
members = [
  "libs/greet/greet.fpasprj",
  "apps/hello/hello.fpasprj",
  "apps/tests/tests.fpasprj",
]
```

Workspace lists members; **each consumer still declares its own** `[dependencies]`. `fpas check` with no path checks all members; `fpas run` with no path or an explicit `.fpasworkspace` runs the sole `kind = "program"` member.

## CLI quick reference

| Command | Purpose |
|---------|---------|
| `fpas` | Print usage |
| `fpas init project <name>` | Create a formatted runnable project scaffold |
| `fpas init library <name> [--unit <name>]` | Create an exported library scaffold |
| `fpas init workspace <name>` | Create a workspace with a program and consumed library |
| `fpas build [<path>]` | Build project or workspace artifacts |
| `fpas build --executable [--name <name>] <path>` | Bundle exactly one program for the current host |
| `fpas run` | Discover and run program in cwd (workspace or single `.fpasprj`) |
| `fpas run <file.fpas>` | Run single program file |
| `fpas run <file.fpasprj>` | Produce/reuse its `.fpascp` and run it |
| `fpas run <file.fpasworkspace>` | Run its sole program member |
| `fpas run <file.fpascp>` | Validate and run a compiled program without sources |
| `fpas check [<path>]` | Type-check without running (`.fpas`, dir, `.fpasprj`, `.fpasworkspace`) |
| `fpas test [<path>]` | Run `*_test.fpas` entries; discover `kind = "test"` in workspace |
| `fpas fmt [<path>]` | Format `.fpas` sources |
| `fpas fmt --check <path>` | Verify formatting |

Useful `fpas test` flags: `--list`, `--fail-fast`, `--strict`, `--filter <pattern>`, `--timeout <secs>`, `--jobs <n>`.

Sidecars (optional, beside a test file): `<test>.expect.stdout`, `<test>.expect.screen`, `<test>.expect.pixels`, `<test>.script.toml`.

Full CLI spec: [`docs/pascal/program-structure/cli.md`](../../../docs/pascal/program-structure/cli.md). Test runner: [`docs/pascal/std/testing/test.md`](../../../docs/pascal/std/testing/test.md).

## Typical workflows

```text
fpas init project my-app             # create project + formatted program source
fpas init library greet --unit Demo.Greet
fpas init workspace my-suite         # create linked program + library members
fpas init project my-app --dry-run --report json
fpas check my-app/my-app.fpasprj   # type-check initialized project
fpas build my-app/my-app.fpasprj   # produce or reuse my-app.fpascp
fpas build --executable my-app/my-app.fpasprj # produce native app / app.exe
fpas run my-app/my-app.fpasprj     # produce/reuse and run my-app.fpascp
fpas run my-app/my-app.fpascp      # run image without project sources
fpas test tests/                   # full regression tree
fpas test tests/suite.fpasprj      # bundled manifest
fpas test tests/stdlib/tui/mvu_host_signature_test.fpas
fpas check                         # sole workspace or project in cwd
```

Discovery with no path:

1. If exactly one `.fpasworkspace` in cwd → use it.
2. Else if exactly one `.fpasprj` → use it.
3. Multiple matches → error; pass an explicit path.

`fpas run` does not accept a directory; use a `.fpas`, `.fpasprj`,
`.fpasworkspace`, or `.fpascp` path.

## Adding a new test to the suite

1. Write `*_test.fpas` under `tests/<theme>/` (`fpas-authoring` skill).
2. Ensure `tests/suite.fpasprj` `[sources].include` covers the path (extend glob if needed).
3. Run `fpas test <new-file>` then `fpas test tests/` or `cargo test -p fpas-cli fpas_suite_`.

## Common mistakes

| Mistake | Fix |
|---------|-----|
| Unit in `.fpasprj` but not imported | Add `uses MyUnit` in the consumer |
| Cyclic `dependencies.projects` | Break the cycle — loader rejects cycles |
| `program` in `kind = "library"` project | Libraries contain `unit` files only |
| `*_test.fpas` in `examples/` | Move to `tests/` |
| Expect workspace to imply deps | Declare `[dependencies]` on each consumer `.fpasprj` |

## When done

- Source files touched → `fpas fmt --check` on changed paths
- Behavior changed → `fpas-change-checklist`
- New `.fpas` content → `fpas-authoring`
