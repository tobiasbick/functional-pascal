# Functional Pascal

A function-first programming language built on Pascal's readable syntax and clean structure. Runs on a managed virtual machine — no pointers, no manual memory management.

Language specification: [`docs/pascal/`](.) — implemented behavior and stdlib reference.

Planned features: [`docs/future/`](../future/).

## Design Principles

- **Function First** — Functions are the primary abstraction. No classical classes or object hierarchies.
- **Immutable by Default** — Variables require `mutable var` block to allow reassignment.
- **Safe by Design** — No pointers, no unsafe operations. The VM handles memory.
- **Explicit Types** — Every binding explicitly states its type.
- **Case-Insensitive** — Keywords and identifiers are case-insensitive, as in classical Pascal.
- **Familiar Syntax** — Pascal's `begin`, `end`, `:=`, `downto`, and other well-known constructs.

## Documentation areas

Topic-based layout (Microsoft Learn style).

| Area | Hub | Topics |
|------|-----|--------|
| Getting started | [getting-started/](getting-started/README.md) | Overview, hello world, keywords |
| Language | [language/](language/README.md) | Types, basics, control flow, functions, pattern matching, errors, concurrency |
| Program structure | [program-structure/](program-structure/README.md) | Units, projects, CLI, workspaces |
| Standard library | [std/](std/README.md) | `Std.*` reference |
| Tools | [tools/](tools/README.md) | `fpas fmt` |
| Formal grammar | [../specs/grammar.ebnf](../specs/grammar.ebnf) | Lexer and parser syntax |

## Start here

Ordered learning path for newcomers:

1. [Getting started](getting-started/README.md) — philosophy, hello world, first program
2. [Basics](language/basics/README.md) — primitives, variables, operators
3. [Control flow](language/control-flow/README.md) — `if`, loops, `case` intro
4. [Functions](language/functions/README.md) — routines, first-class calls
5. [Types](language/types/README.md) — records, enums, arrays, generics
6. [Pattern matching](language/pattern-matching/README.md) — guards, exhaustiveness
7. [Error handling](language/error-handling/README.md) — `Result`, `Option`, `try`, `panic`
8. [Concurrency](language/concurrency/README.md) — `go`, tasks, fork-join
9. [Units](program-structure/units.md) — `uses`, namespaces, visibility
10. [Projects](program-structure/projects.md) — `.fpasprj`, workspaces
11. [CLI](program-structure/cli.md) — `fpas`, `check`, `test`, `fmt`
12. [Standard library](std/README.md) — `Std.*` built-in units
13. [Formatter style](tools/fmt-style.md) — normative rules for `fpas fmt`

## Future Features

Features planned for later versions are documented in [docs/future/](../future/).
