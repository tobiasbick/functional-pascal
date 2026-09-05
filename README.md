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
- **Standard library** — Built-in `Std.*` units for console I/O, TUI, strings, math, arrays, tasks, and more.
- **Editor support** — A repository-owned VS Code-compatible extension provides diagnostics, formatting, navigation, completion, project workflows, integrated tests, and source debugging.
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

The compiler is at `target/release/fpas` (or `fpas.exe` on Windows). Keep the
adjacent `target/release/lib/` source standard library and
`target/release/fpas-runner` (or `fpas-runner.exe`) with it. The runner is used
by `fpas build --executable`; generated applications themselves are standalone.
Run the compiler directly or add the whole `target/release` directory to your
`PATH`.

To prepare the distributable layout under `bin/`, run `./dist.sh` on Linux or
`./dist.ps1` on Windows. Both scripts copy `fpas`, `fpas-runner`, and `lib/`
together.
Both scripts resolve paths from their own directory and stop on build,
standard-library staging, or copy failures. They report success only after all
steps finish.

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

Run `fpas fmt` manually for command-line workflows. The repository's
[VS Code-compatible editor integration](docs/pascal/tools/editor-integration.md)
also provides diagnostics, **Format Document**, document symbols, hover,
definition, references, rename, and basic completion. Its bounded project catalog follows
manifest dependencies and refreshes navigation after watched source or manifest changes. It
works with the editor's standard format-on-save setting; no FPAS watch mode is required.

### VS Code-compatible extension

With Node.js 22 or newer and a stable Rust toolchain installed, build the local
host-native VSIX from the repository root:

```sh
npm ci --prefix editors/vscode
npm run package --prefix editors/vscode
```

The result is
`editors/vscode/dist/functional-pascal-<version>-<host-target>.vsix`. Install
it in VS Code, Cursor, VSCodium, or another compatible desktop editor through
**Extensions: Install from VSIX**. The package includes the native language
server for the host where it was built; users on another operating system or
architecture build it there.

## Applications

[Local Chat](apps/local-chat/README.md) is a small `Std.Tui` client for the
fixed local OpenAI-compatible llama.cpp endpoint.

```sh
fpas run apps/local-chat/local-chat.fpasprj
```

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
  WriteLn(Apply(Op, 10)) // 20
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

Author-facing tests are `*_test.fpas` programs under [`tests/`](tests/) (`stdlib/`, including `stdlib/tui/`, `concurrency/`, `runner/`, `console/`, and `apps/`). Run the full suite with `fpas test tests/` or `fpas test tests/suite.fpasprj`. See [`docs/pascal/std/testing/test.md`](docs/pascal/std/testing/test.md) and [`examples/README.md`](examples/README.md).

### Local model training data

The reproducible Functional Pascal fine-tuning dataset lives under
[`training/fpas/`](training/fpas/). It is generated from implemented docs,
examples, applications, and regression tests and uses Hugging Face's
conversational JSONL format. Build and validate it locally with:

```sh
python training/fpas/generate_dataset.py
python training/fpas/validate_dataset.py
```

The generated files can be selected in Unsloth Studio for local LoRA/QLoRA
training. Keep the held-out test split out of training and validate generated
FPAS with the compiler and test runner afterward.

### Multi-file projects and libraries

Larger programs use a `.fpasprj` project file. Each imported unit is built independently into a source-adjacent `.fpascu` sidecar and linked into the final program automatically. Sources and manifests remain authoritative; sidecars are derived, Git-ignored build outputs. Reference library projects from `[dependencies].projects` (paths) or `[dependencies].workspace` (member `project.name` inside a `.fpasworkspace`). Libraries may hide internal units from dependents with `[exports].units` in the library `.fpasprj`. See [Projects](docs/pascal/program-structure/projects.md), [library-deps](examples/pascal/library-deps/), and [monorepo](examples/pascal/monorepo/).

```sh
fpas init project my-app
fpas init library my-lib --unit MyLib
fpas init workspace my-suite
fpas run my-app/my-app.fpasprj
fpas check my-lib/my-lib.fpasprj
fpas check my-suite/my-suite.fpasworkspace   # check every workspace member
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
| Standard library | [std/](docs/pascal/std/README.md) — themed subdirs (`host/`, `text/str/`, `console/`, `tui/`, `network/`, …) |
| Tools | [tools/](docs/pascal/tools/README.md) — formatter and editor integration |
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
14. [Editor integration](docs/pascal/tools/editor-integration.md)

Planned work (not current behavior): [`docs/future/`](docs/future/).

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md). Short pointers:

- Language spec: [`docs/pascal/`](docs/pascal/) ([hub](docs/pascal/README.md)) — source of truth for implemented behavior
- Agents: [`AGENTS.md`](AGENTS.md) and skills under [`.agents/skills/`](.agents/skills/)
- Examples: [`examples/README.md`](examples/README.md)
- FPAS tests: [`tests/`](tests/) and [`docs/pascal/std/testing/test.md`](docs/pascal/std/testing/test.md)
- Verify locally: `cargo fmt`, `cargo build`, `cargo test --workspace`, and `fpas fmt --check` on touched `.fpas` paths when relevant

## Project Structure

| Component | Purpose |
|-----------|---------|
| `fpas-cli` | Command-line interface (`fpas` binary) |
| `fpas-lexer` | Tokenizer / lexical analysis |
| `fpas-parser` | Parser producing the AST |
| `fpas-project` | Project/workspace loading and unit-graph resolution |
| `fpas-build` | Incremental compiled-unit build engine |
| `fpas-sema` | Semantic analysis and type checking |
| `fpas-ir` | Typed control-flow intermediate representation |
| `fpas-compiler` | AST lowering through typed IR to register bytecode |
| `fpas-bytecode` | Register-bytecode definitions and executable verification |
| `fpas-unit` | Compiled-unit identities, format, and sidecar lifecycle |
| `fpas-linker` | Deterministic linker from unit objects to verified executables |
| `fpas-program` | Persistent executable `.fpascp` program images |
| `fpas-bundle` | Host-native runner bundle format and publication |
| `fpas-vm` | Virtual machine / bytecode interpreter |
| `fpas-std` | Standard library intrinsics |
| `fpas-fmt` | Canonical FPAS source formatter |
| `fpas-diagnostics` | Error codes and diagnostic utilities |
| `fpas-language-service` | Compiler-backed editor analysis and language features |
| `fpas-lsp` | Language Server Protocol transport |
| `fpas-debug` | Source debug engine with JSONL and DAP adapters |
| `fpas-bench` | Bounded performance harness, baselines, and comparisons |
| `editors/vscode` | VS Code-compatible extension, packaging, and Extension Host tests |

## Status

**v0.0.1 — Experimental.** The language specification and compiler are under active development. Expect breaking changes.

## License

[BSD-3-Clause](LICENSE) © 2026 Tobias Bick
