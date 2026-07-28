# Functional Pascal

A modern, function-first programming language built on Pascal's readable syntax. Compiles `.fpas` source files to bytecode and runs them on a managed virtual machine.

> **⚠️ Disclaimer:** This is a small hobby project, entirely vibe-coded. It started as an experiment in learning how to effectively communicate and collaborate with LLMs. The future is uncertain — no idea where this will end up, or if it will.

[![License: BSD-3-Clause](https://img.shields.io/badge/License-BSD--3--Clause-blue.svg)](LICENSE)

## Features

- **Function-first** — Functions are the primary building block. No classical classes.
- **Immutable by default** — All bindings are immutable unless declared with `mutable var`.
- **Pattern matching** — Exhaustive `case` statements with enum, `Result`, and `Option` destructuring.
- **First-class functions** — Pass named functions as values, store them in variables, and use them with higher-order APIs.
- **Error handling** — Built-in `Result of T, E` and `Option of T` types with a `try` operator for propagation.
- **Concurrency** — Go-inspired `go` tasks with `Wait` and `WaitAll` for fork-join concurrency.
- **Standard library** — Built-in `Std.*` units for console I/O, TUI, native graphics, strings, math, arrays, tasks, and more.
- **Safe by design** — The VM manages memory. No pointers, no manual allocation, no unsafe operations.
- **Case-insensitive** — Keywords and identifiers are case-insensitive, following Pascal tradition.
- **Explicit types** — Every variable and parameter declares its type.

## Quickstart

### Build from source

```sh
git clone https://github.com/tobiasbick/functional-pascal.git
cd functional-pascal
cargo build --release
```

The executable is at `target/release/fpas` (or `fpas.exe` on Windows). Keep the adjacent `target/release/lib/` directory with it: it contains the bundled source standard library. Run the executable directly or add the whole `target/release` directory to your `PATH`.

To prepare the distributable layout under `bin/`, run `./dist.sh` on Unix or `./dist.ps1` on Windows. Both scripts copy the executable and `lib/` together.

Alternatively, run without installing:

```sh
cargo run -p fpas-cli -- run examples/hello.fpas
```

### Hello World

Create `hello.fpas`:

```pascal
program Hello;
uses Std.Console;
begin
  WriteLn('Hello, World!')
end.
```

Run it:

```sh
fpas run hello.fpas
```

### Formatting

`.fpas` sources under `examples/`, `tests/`, and `apps/` follow the official formatter style ([`docs/pascal/tools/fmt-style.md`](docs/pascal/tools/fmt-style.md)). Format in place:

```sh
scripts/format-fpas-sources.sh          # Unix
scripts/format-fpas-sources.ps1         # Windows
# or: cargo run -p fpas-cli -- fmt examples tests apps
```

Check without writing:

```sh
cargo run -p fpas-cli -- fmt --check examples tests apps
```

Run `fpas fmt` manually when you want to apply formatting — there is no format-on-save or watch mode.

## Applications

[Notes](apps/notes/README.md) is a complete modern `Std.Tui` note-taking
application with local human-readable `.note` files, responsive layouts,
keyboard and mouse control, a command palette, and headless workflow tests.

```sh
fpas run apps/notes/notes.fpasprj -- ./my-notes
```

See [`apps/`](apps/README.md) for complete applications. Focused language and
standard-library demonstrations remain under `examples/`.

## Examples

### Fibonacci

```pascal
program Fibonacci;
uses Std.Console;

function Fib(N: integer): integer;
begin
  if N <= 1 then
    return N
  else
    return Fib(N - 1) + Fib(N - 2)
end;

begin
  WriteLn('Fibonacci sequence:');
  for I: integer := 0 to 9 do
    WriteLn(Fib(I))
end.
```

### Pattern Matching

```pascal
program PatternMatching;
uses Std.Console;

type
  Light = enum
    Red;
    Yellow;
    Green;
  end;

function TrafficAdvice(L: Light): string;
begin
  case L of
    Light.Red:    return 'Stop';
    Light.Yellow: return 'Caution';
    Light.Green:  return 'Go'
  end
end;

begin
  WriteLn(TrafficAdvice(Light.Red))
end.
```

### Higher-Order Functions

```pascal
program HigherOrderFunctions;
uses Std.Console;

function Double(X: integer): integer;
begin
  return X * 2
end;

function Apply(F: function(X: integer): integer; Value: integer): integer;
begin
  return F(Value)
end;

begin
  var Op: function(X: integer): integer := Double;
  WriteLn(Apply(Op, 10));  { 20 }
end.
```

### Error Handling with Option

```pascal
program OptionExample;
uses Std.Console, Std.Array;

function FindFirst(Items: array of integer; Min: integer): Option of integer;
begin
  for I: integer := 0 to Length(Items) - 1 do
    if Items[I] >= Min then
      return Some(Items[I]);
  return None
end;

begin
  case FindFirst([3, 7, 15, 42], 10) of
    Some(V): WriteLn('Found: ', V);
    None:    WriteLn('Not found')
  end
end.
```

More examples in the [`examples/`](examples/) directory.

### Tests

Author-facing tests are `*_test.fpas` programs under [`tests/`](tests/) (`stdlib/`, including `stdlib/tui/`, `concurrency/`, `runner/`, `console/`, and `graph/`). Run the full suite with `fpas test tests/` or `cargo test -p fpas-cli fpas_suite_`. See [`docs/pascal/std/testing/test.md`](docs/pascal/std/testing/test.md) and [`examples/README.md`](examples/README.md).

### Multi-file projects and libraries

Larger programs use a `.fpasprj` project file. Each imported unit is built independently into a source-adjacent `.fpascu` sidecar and linked into the final program automatically. Sources and manifests remain authoritative; sidecars are derived, Git-ignored build outputs. Reference library projects from `[dependencies].projects` (paths) or `[dependencies].workspace` (member `project.name` inside a `.fpasworkspace`). Libraries may hide internal units from dependents with `[exports].units` in the library `.fpasprj`. See [Projects](docs/pascal/program-structure/projects.md), [library-deps](examples/pascal/library-deps/), and [monorepo](examples/pascal/monorepo/).

```sh
fpas run my-app.fpasprj
fpas check my-lib.fpasprj
fpas check my-suite.fpasworkspace   # check every workspace member
cd my-suite && fpas check           # discover .fpasworkspace in cwd
cd my-suite && fpas run             # run the sole program member
```

## Documentation

The full language specification lives in [`docs/pascal/`](docs/pascal/). Start with the [documentation hub](docs/pascal/README.md) for area navigation and the learning path.

| Area | Hub |
|------|-----|
| Getting started | [getting-started/](docs/pascal/getting-started/README.md) |
| Language | [language/](docs/pascal/language/README.md) |
| Program structure | [program-structure/](docs/pascal/program-structure/README.md) |
| Standard library | [std/](docs/pascal/std/README.md) — themed subdirs (`host/`, `text/str/`, `console/`, `tui/`, `graph/app/`, …) |
| Tools | [tools/](docs/pascal/tools/README.md) |
| Formal grammar | [grammar.ebnf](docs/specs/grammar.ebnf) |

Ordered learning path:

1. [Overview](docs/pascal/getting-started/README.md)
2. [Basics](docs/pascal/language/basics/README.md)
3. [Control Flow](docs/pascal/language/control-flow/README.md)
4. [Functions](docs/pascal/language/functions/README.md)
5. [Types](docs/pascal/language/types/README.md)
6. [Pattern Matching](docs/pascal/language/pattern-matching/README.md)
7. [Error Handling](docs/pascal/language/error-handling/README.md)
8. [Concurrency](docs/pascal/language/concurrency/README.md)
9. [Units](docs/pascal/program-structure/units.md)
10. [Projects](docs/pascal/program-structure/projects.md)
11. [CLI](docs/pascal/program-structure/cli.md)
12. [Standard Library](docs/pascal/std/README.md)
13. [Formatter style](docs/pascal/tools/fmt-style.md)

Planned work (not current behavior): [`docs/future/`](docs/future/).

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md). Short pointers:

- Language spec: [`docs/pascal/`](docs/pascal/) ([hub](docs/pascal/README.md)) — source of truth for implemented behavior
- Agents: [`AGENTS.md`](AGENTS.md) and skills under [`.agents/skills/`](.agents/skills/)
- Examples: [`examples/README.md`](examples/README.md)
- FPAS tests: [`tests/`](tests/) and [`docs/pascal/std/testing/test.md`](docs/pascal/std/testing/test.md)
- Verify locally: `cargo fmt`, `cargo build`, `cargo test --workspace`, and `fpas fmt --check` on touched `.fpas` paths when relevant

## Project Structure

| Crate | Purpose |
|-------|---------|
| `fpas-cli` | Command-line interface (`fpas` binary) |
| `fpas-lexer` | Tokenizer / lexical analysis |
| `fpas-parser` | Parser producing the AST |
| `fpas-project` | Project/workspace loading and unit-graph resolution |
| `fpas-build` | Incremental compiled-unit build engine |
| `fpas-sema` | Semantic analysis and type checking |
| `fpas-compiler` | AST-to-bytecode compilation |
| `fpas-bytecode` | Bytecode definitions and chunk format |
| `fpas-unit` | Compiled-unit identities, format, and sidecar lifecycle |
| `fpas-linker` | Deterministic linker from unit objects to executable chunks |
| `fpas-vm` | Virtual machine / bytecode interpreter |
| `fpas-std` | Standard library intrinsics |
| `fpas-fmt` | Canonical FPAS source formatter |
| `fpas-diagnostics` | Error codes and diagnostic utilities |

## Status

**v0.0.1 — Experimental.** The language specification and compiler are under active development. Expect breaking changes.

## License

[BSD-3-Clause](LICENSE) © 2026 Tobias Bick
