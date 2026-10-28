# Functional Pascal

A functional-first Pascal language with selected Ada conventions. Compiles `.fpas` source files to bytecode and runs them on a managed virtual machine.

> **⚠️ Disclaimer:** This is a small hobby project, entirely vibe-coded. It started as an experiment in learning how to effectively communicate and collaborate with LLMs. The future is uncertain — no idea where this will end up, or if it will.

[![License: BSD-3-Clause](https://img.shields.io/badge/License-BSD--3--Clause-blue.svg)](LICENSE)

## Features

- **Function-first** — Functions are the primary building block. No classical classes.
- **Explicit binding mutability** — `const` bindings are immutable; `var` bindings permit reassignment. Parameters are read-only unless declared with `var`, which also marks the caller's argument.
- **Pattern matching** — Exhaustive `case` statements with enum, `Result`, and `Option` destructuring.
- **First-class functions** — Pass named functions as values, store them in variables, and use them with higher-order APIs.
- **Contracts** — Declare routine preconditions with `requires` and postconditions with `ensures`. Checks run by default in every build mode; [`--no-contracts`](docs/pascal/program-structure/cli.md#contract-policy) omits their execution while retaining type checking.
- **Generic records and enums** — Declare type parameters with `of` and infer type arguments from supplied constructor fields or payloads.
- **Subranges and distinct types** — Restrict integer values with checked bounds and give scalar domain types their own identity.
- **Error handling** — Built-in `Result of (T, E)` and `Option of T`: `try` propagates, nonbinding `is` tests variants, and exhaustive `case` handles payloads and defaults. [Std.Text.Parse](docs/pascal/std/text/parse.md) text parsers return `Result`; `Unwrap` deliberately aborts on failure.
- **Concurrency** — `go` tasks, typed bounded channels, cooperative cancellation, mixed-source `Select`, task groups, and supervised retries. [Std.Tasks](docs/pascal/std/concurrency/task.md) includes timed group close; timeout retains unfinished work rather than forcibly terminating it.
- **Task scopes** — `scope ... end scope;` joins owned child tasks before exit, supports cooperative cancellation, and propagates runtime failures. [Scope-owned handles](docs/pascal/language/concurrency/task-scopes.md#handle-escapes) stay inside their scope.
- **Standard library** — Built-in `Std.*` units for console I/O, TUI, math, tasks, files, networking, and more. Strings, arrays, dictionaries, `Option`, and `Result` have native operations that need no imports.
- **Visible mutation** — Native collection and string transformations return values; `InPlace` operations update writable receivers.
- **Server lifetime** — [Std.Net.Server](docs/pascal/std/network/server.md) owns worker groups and listeners, handles stop signals, and supports explicitly authorized process escalation after a common grace period.
- **Editor support** — A repository-owned VS Code-compatible extension provides diagnostics, formatting, navigation, completion, project workflows, integrated tests, and source debugging.
- **Safe by design** — The VM manages memory. No pointers, no manual allocation, no unsafe operations.
- **Case-insensitive** — Keywords and identifiers are case-insensitive, following Pascal tradition.
- **Explicit types** — Parameters, routine results, and record fields declare their types. Statement-local bindings support [type inference](docs/pascal/language/basics/local-variables.md#type-inference) whenever the initializer's type is determined, including calls and `if`/`case` expressions.

## Quickstart

### Build from source

Rust 1.99.1 or newer is required. Install Rust through [rustup](https://rustup.rs/);
the repository's `rust-toolchain.toml` selects Rust 1.99.1 with rustfmt and Clippy
for reproducible development builds.

```sh
git clone https://github.com/tobiasbick/functional-pascal.git
cd functional-pascal
cargo build --release
```

The compiler is at `target/release/fpas` (or `fpas.exe` on Windows). Keep the
adjacent `target/release/lib/` source standard library and
`target/release/fpas-runner` (or `fpas-runner.exe`) with it. The runner is used
by `fpas build --executable`; generated applications themselves are standalone.
Editor integration starts the adjacent `fpas-lsp` (`fpas-lsp.exe` on Windows).
CLI, language server, and runner are separate Cargo packages in this workspace.
Run the compiler directly or add the whole `target/release` directory to your
`PATH`.

To prepare the distributable layout under `bin/`, run `./dist.sh` on Linux or
`./dist.ps1` on Windows. Both scripts copy `fpas`, `fpas-runner`, `fpas-lsp`,
and `lib/` together. They build the Rust binaries in one Cargo invocation, then
use the built compiler's `stdlib stage` command to precompile and stage `lib/`.
Both scripts resolve paths from their own directory and stop on build,
standard-library staging, or copy failures. They report success only after all
steps finish.

Release builds, `cargo bench-fpas`, and both dist scripts use Fat-LTO. For
development, use the optimized `dev-fast` profile with Thin-LTO:

```sh
cargo build --profile dev-fast
```

Development binaries and their adjacent `lib/` are in `target/dev-fast/`.
The [build guide](docs/bench/build-times.md) describes the profiles, dependency
boundaries, and how to measure rebuilds.

Alternatively, run a development build without installing:

```sh
cargo run --profile dev-fast -p fpas-cli -- run examples/hello.fpas
```

### Hello World

Create `hello.fpas`:

```pascal id=example-3
program Hello;

uses Std.Console as Console;

begin
  Console.WriteLn('Hello, World!');
end.
```

Run it:

```sh
fpas run hello.fpas
```

### Formatting

`.fpas` sources under `examples/`, `tests/`, and `apps/` follow the official formatter style ([`docs/pascal/tools/fmt-style.md`](docs/pascal/tools/fmt-style.md)). Format in place:

```sh
bash scripts/format-fpas-sources.sh     # Unix
scripts/format-fpas-sources.ps1         # Windows
# include standard-library sources: cargo run -p fpas-cli -- fmt examples tests apps lib
```

Check without writing:

```sh
cargo run -p fpas-cli -- fmt --check examples tests apps lib
```

The CLI and editor use one fixed style: two-space structural indentation and a
100-column target for expressions. The formatter keeps authored line groups,
expands long calls and collections automatically, and keeps postfix chains
attached. Multiline statement conditions put `then` on its own line; wrapped
`and`, `or`, and `xor` also occupy separate continuation lines.

Run `fpas fmt` manually for command-line workflows. The repository's
[VS Code-compatible editor integration](docs/pascal/tools/editor-integration.md)
also provides diagnostics, **Format Document**, document symbols, hover,
definition, references, rename, completion, signature help, and import quick fixes. Its bounded project catalog follows
manifest dependencies and refreshes navigation after watched source or manifest changes. It
works with the editor's standard format-on-save setting; no FPAS watch mode is required.

### VS Code-compatible extension

With Node.js 22 or newer installed, build the local
toolchain-independent VSIX from the repository root:

```sh
npm ci --prefix editors/vscode
npm run package --prefix editors/vscode
```

This compiles the extension and verifies the archive without building FPAS or
running integration tests. To run the full extension test suite before packaging,
use `npm run package:verified --prefix editors/vscode`; that command also requires
the repository's pinned Rust toolchain.

The result is `editors/vscode/dist/functional-pascal-<version>.vsix`. Install
it in VS Code, Cursor, VSCodium, or another compatible desktop editor through
**Extensions: Install from VSIX**. Install the FPAS toolchain separately and
put `fpas` on `PATH`, or select it through the extension's
`functionalPascal.executablePath` setting. The same selected toolchain supplies
the compiler, language server, debugger, and standard library.

## Applications

[Local Chat](apps/local-chat/README.md) is a small `Std.Ui.Terminal` client for the
fixed local OpenAI-compatible llama.cpp endpoint.

```sh
fpas run apps/local-chat/local-chat.fpasprj
```

[Notes](apps/notes/README.md) is a complete modern `Std.Ui.Terminal` note-taking
application with local human-readable `.note` files, responsive layouts,
keyboard and mouse control, a command palette, and headless workflow tests.

```sh
fpas run apps/notes/notes.fpasprj -- ./my-notes
```

See [`apps/`](apps/README.md) for complete applications. Focused language and
standard-library demonstrations remain under `examples/`.

## Examples

### Fibonacci

```pascal id=example-10
program Fibonacci;

uses Std.Console as Console;

function Fib(N: integer): integer;
begin
  if N <= 1 then
    return N;
  else
    return Fib(N - 1) + Fib(N - 2);
  end if;
end function;

begin
  Console.WriteLn('Fibonacci sequence:');
  for I: integer := 0 to 9 do
    Console.WriteLn(Fib(I));
  end for;
end.
```

### Pattern Matching

```pascal id=example-11
program PatternMatching;

uses Std.Console as Console;

type Light = enum
  Red;
  Yellow;
  Green;
end enum;

function TrafficAdvice(L: Light): string;
begin
  case L of
    when Light.Red:
      return 'Stop';
    when Light.Yellow:
      return 'Caution';
    when Light.Green:
      return 'Go';
  end case;
end function;

begin
  Console.WriteLn(TrafficAdvice(Light.Red));
end.
```

### Higher-Order Functions

```pascal id=example-12
program HigherOrderFunctions;

uses Std.Console as Console;

function Double(X: integer): integer;
begin
  return X * 2;
end function;

function Apply(F: function(X: integer): integer; Value: integer): integer;
begin
  return F(Value);
end function;

begin
  const Op: function(X: integer): integer := Double;
  Console.WriteLn(Apply(Op, 10)); // 20
end.
```

### Error Handling with Option

```pascal id=example-13
program OptionExample;

uses Std.Console as Console;

function FindFirst(Items: array of integer; Min: integer): option of integer;
begin
  for I: integer := 0 to Items.Length() - 1 do
    if Items[I] >= Min then
      return Some(Items[I]);
    end if;
  end for;

  return None;
end function;

begin
  case FindFirst([3, 7, 15, 42], 10) of
    when Some(const V):
      begin
        Console.WriteLn('Found: ', V);
      end;
    when None:
      begin
        Console.WriteLn('Not found');
      end;
  end case;
end.
```

### Parsing and error propagation

Import `Std.Text.Parse` through an alias. Its `Int`, `Real`, `Bool`, and `HexInt`
operations take `Text: string` and return `Result of (T, Parse.ParseError)`. Use `try`
inside a compatible function and `case` to handle the result locally:

```pascal id=example-14
program ParseExample;

uses Std.Text.Parse as Parse, Std.Console as Console;

function DoubleInput(Text: string): result of (integer, Parse.ParseError);
begin
  const Value := try Parse.Int(Text := Text);
  return Ok(Value * 2);
end function;

begin
  for Text: string in ['21', 'invalid'] do
    case DoubleInput(Text := Text) of
      when Ok(const Value):
        Console.WriteLn('Doubled: ', Value);
      when Error(const Cause):
        Console.WriteLn('Input error: ', Cause.Message());
    end case;
  end for;
end.
```

More examples in the [`examples/`](examples) directory.

### Interactive Mandelbrot TUI

The canonical interactive `Std.Ui.Terminal` example renders complete Mandelbrot images
from a cancellable subscription using the VM worker pool. Navigation replaces the active
generation without blocking terminal input, and only the serialized TUI host
updates the model and paints frames.

```sh
fpas run examples/math/mandelbrot/mandelbrot.fpasprj
```

See the [Mandelbrot example](examples/math/mandelbrot/README.md) for controls
and its background-work structure.

### Tests

Author-facing tests are `*_test.fpas` programs under [`tests/`](tests) (`stdlib/`, including `stdlib/tui/`, `concurrency/`, `runner/`, `console/`, and `apps/`). Run the full suite with `fpas test tests/` or `fpas test tests/suite.fpasprj`. See [`docs/pascal/std/testing/test.md`](docs/pascal/std/testing/test.md) and [`examples/README.md`](examples/README.md).

The suite also checks every Pascal example in this README and `docs/pascal/`.
The [documentation example checker](docs/pascal/tools/documentation-examples.md)
is written in FPAS. It checks fragments with explicit contexts and executes only
examples marked `run`. Use `fpas run tools/doc-examples/doc-examples.fpasprj` to
keep the generated sources and report under `.temp-data/` for inspection.

### Multi-file projects and libraries

Larger programs use a `.fpasprj` project file. Each imported unit is built independently into a source-adjacent `.fpascu` sidecar and linked into the final program automatically. Sources and manifests remain authoritative; sidecars are derived, Git-ignored build outputs. Reference library projects from `[dependencies].projects` (paths) or `[dependencies].workspace` (member `project.name` inside a `.fpasworkspace`). Libraries may hide internal units from dependents with `[exports].units` in the library `.fpasprj`. See [Projects](docs/pascal/program-structure/projects.md), [library-deps](examples/pascal/library-deps), and [monorepo](examples/pascal/monorepo).

```sh
fpas init project my-app
fpas init library my-lib --unit MyLib
fpas init workspace my-suite
fpas run my-app/my-app.fpasprj
fpas check my-lib/my-lib.fpasprj
fpas check my-suite/my-suite.fpasworkspace   # check every workspace member
cd my-suite
fpas check                         # discover .fpasworkspace in cwd
fpas run                           # run the sole program member
```

## Documentation

The full language specification lives in [`docs/pascal/`](docs/pascal). Start with the [documentation hub](docs/pascal/README.md) for area navigation and the learning path.

For authoring with an LLM, start with the [FPAS reference for agents](docs/pascal/agents/README.md).
The short entrypoint links to focused language topics. Its
[maintenance guide](docs/pascal/agents/maintenance.md) defines the total token
budget and how to keep the rules and executable examples current.

For runtime failures, `fpas run app.fpasprj --failure-dump .temp-data/failure.json`
retains a [bounded JSON report](docs/pascal/tools/failure-dump.md) with the original
error, task states, frames and values. Choose a fresh report path for each run.

| Area | Hub |
|------|-----|
| Getting started | [getting-started/](docs/pascal/getting-started/README.md) |
| Language | [language/](docs/pascal/language/README.md) |
| Program structure | [program-structure/](docs/pascal/program-structure/README.md) |
| Standard library | [std/](docs/pascal/std/README.md) — themed subdirs (`host/`, `text/`, `console/`, `tui/`, `network/`, …) |
| Tools | [tools/](docs/pascal/tools/README.md) — declaration lookup, source queries, diagnostics, debugging, profiling, formatting, and editor integration |
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

Roadmaps, implementation progress, and deferred work: [`docs/future/`](docs/future).

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md). Short pointers:

- Language spec: [`docs/pascal/`](docs/pascal) ([hub](docs/pascal/README.md)) — source of truth for implemented behavior
- Agents: [`AGENTS.md`](AGENTS.md)
- Examples: [`examples/README.md`](examples/README.md)
- FPAS tests: [`tests/`](tests) and [`docs/pascal/std/testing/test.md`](docs/pascal/std/testing/test.md)
- Verify locally: `cargo fmt`, `cargo build`, `cargo test --workspace`, and `fpas fmt --check` on touched `.fpas` paths when relevant

## Project Structure

| Component | Purpose |
|-----------|---------|
| `fpas-cli` | Command-line interface (`fpas` binary) |
| `fpas-lexer` | Tokenizer / lexical analysis |
| `fpas-parser` | Parser producing the AST |
| `fpas-numeric` | Shared numeric parsing, arithmetic, and conversion rules |
| `fpas-project` | Project/workspace loading and unit-graph resolution |
| `fpas-build` | Incremental compiled-unit build engine |
| `fpas-sema` | Semantic analysis and type checking |
| `fpas-ir` | Typed control-flow intermediate representation |
| `fpas-compiler` | AST lowering through typed IR to register bytecode |
| `fpas-code` | Portable bytecode, executable metadata, and validation |
| `fpas-builtins` | Standard-library names, native declarations, and shared discriminants |
| `fpas-runtime` | VM execution, values, memory ownership, native operations, and host integration |
| `fpas-binary` | Shared little-endian encoding primitives and BLAKE3 digests |
| `fpas-unit` | Compiled-unit identities, format, and sidecar lifecycle |
| `fpas-linker` | Deterministic linker from unit objects to verified executables |
| `fpas-program` | Persistent executable `.fpascp` program images |
| `fpas-bundle` | Host-native runner bundle format and publication |
| `fpas-runner` | Standalone runtime executable for bundled programs |
| `fpas-fmt` | Canonical FPAS source formatter |
| `fpas-diagnostics` | Error codes and diagnostic utilities |
| `fpas-language-service` | Compiler-backed editor analysis and language features |
| `fpas-lsp` | Standalone Language Server Protocol executable and transport |
| `fpas-debug` | Source debug engine with batch, JSONL and DAP adapters |
| `fpas-bench` | Bounded performance harness, baselines, and comparisons |
| `fpas-acp-model` | ACP schemas, declaration catalog, and protocol data |
| `fpas-acp-codec` | Typed ACP wire conversions and SDK normalization |
| `fpas-acp` | ACP connections, transport, interaction state, and MCP bridge |
| `fpas-mcp` | MCP tool registration and serving |
| `editors/vscode` | VS Code-compatible extension, packaging, and Extension Host tests |

See the [Rust architecture](docs/rust-architecture.md) for dependency direction,
runtime admission, and test boundaries.

## Status

**v0.0.1 — Experimental.** The language specification and compiler are under active development. Expect breaking changes.

## License

[BSD-3-Clause](LICENSE) © 2026 Tobias Bick
