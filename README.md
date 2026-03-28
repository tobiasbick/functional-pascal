# Functional Pascal

A modern, function-first programming language built on Pascal's readable syntax. Compiles `.fpas` source files to bytecode and runs them on a managed virtual machine.

> **⚠️ Disclaimer:** This is a small hobby project, entirely vibe-coded. It started as an experiment in learning how to effectively communicate and collaborate with LLMs. The future is uncertain — no idea where this will end up, or if it will.

[![License: BSD-3-Clause](https://img.shields.io/badge/License-BSD--3--Clause-blue.svg)](LICENSE)

## Features

- **Function-first** — Functions are the primary building block. No classical classes.
- **Immutable by default** — All bindings are immutable unless declared `mutable`.
- **Pattern matching** — Exhaustive `case` expressions with enum, `Result`, and `Option` destructuring.
- **First-class functions & closures** — Pass functions as values, return closures that capture their environment.
- **Error handling** — Built-in `Result of T, E` and `Option of T` types with a `try` operator for propagation.
- **Concurrency** — Go-inspired `go` tasks and typed channels for concurrent programming.
- **Safe by design** — The VM manages memory. No pointers, no manual allocation, no unsafe operations.
- **Case-insensitive** — Keywords and identifiers are case-insensitive, following Pascal tradition.
- **Explicit types** — Every variable and parameter declares its type.

## Quickstart

### Build from source

```sh
git clone https://github.com/ArcticDev/functional-pascal.git
cd functional-pascal
cargo build --release
```

The binary is at `target/release/fpas` (or `fpas.exe` on Windows).

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
```

### Closures

```pascal
function MakeAdder(N: integer): function(X: integer): integer;
begin
  return function(X: integer): integer
  begin
    return X + N
  end
end;

begin
  var Add5: function(X: integer): integer := MakeAdder(5);
  WriteLn(Add5(10));  { 15 }
end.
```

### Error Handling with Option

```pascal
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

## Documentation

The full language documentation is in [`docs/pascal/`](docs/pascal/):

1. [Overview](docs/pascal/01-overview.md)
2. [Basics](docs/pascal/02-basics.md)
3. [Control Flow](docs/pascal/03-control-flow.md)
4. [Functions](docs/pascal/04-functions.md)
5. [Types](docs/pascal/05-types.md)
6. [Pattern Matching](docs/pascal/06-pattern-matching.md)
7. [Error Handling](docs/pascal/07-error-handling.md)
8. [Concurrency](docs/pascal/08-concurrency.md)
9. [Units](docs/pascal/09-units.md)
10. [Projects](docs/pascal/10-projects.md)
11. [Standard Library](docs/pascal/11-stdlib.md)

## Project Structure

| Crate | Purpose |
|-------|---------|
| `fpas-cli` | Command-line interface (`fpas` binary) |
| `fpas-lexer` | Tokenizer / lexical analysis |
| `fpas-parser` | Parser producing the AST |
| `fpas-sema` | Semantic analysis and type checking |
| `fpas-compiler` | AST-to-bytecode compilation |
| `fpas-bytecode` | Bytecode definitions and chunk format |
| `fpas-vm` | Virtual machine / bytecode interpreter |
| `fpas-std` | Standard library intrinsics |
| `fpas-diagnostics` | Error codes and diagnostic utilities |

## Status

**v0.0.1 — Experimental.** The language specification and compiler are under active development. Expect breaking changes.

## License

[BSD-3-Clause](LICENSE) © 2026 Tobias Bick
