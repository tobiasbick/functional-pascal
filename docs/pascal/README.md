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

Topic-based layout (Microsoft Learn style). **Types** is migrated; other areas link to legacy chapters until moved.

| Area | Hub |
|------|-----|
| [Getting started](getting-started/README.md) | Overview, hello world |
| [Language](language/README.md) | Core reference |
| [Types](language/types/README.md) | Records, enums, arrays, generics |
| [Basics](language/basics/README.md) | Primitives, variables, operators |
| [Program structure](program-structure/README.md) | **migrated** | Units, projects, CLI, workspaces |
| [Standard library](std/README.md) | `Std.*` reference |
| [Tools](tools/README.md) | `fpas fmt` |

## Table of Contents (legacy numbered chapters)

1. [Overview](01-overview.md) — Philosophy, hello world, first taste
2. [Basics](language/basics/README.md) — Primitive types, variables, constants, operators *(migrated)*
3. [Control Flow](language/control-flow/README.md) — Conditionals, loops, branching *(migrated)*
4. [Functions](language/functions/README.md) — Functions, procedures, first-class functions, nested functions *(migrated)*
5. [Types](language/types/README.md) — Records, enumerations, arrays, type aliases *(migrated)*
6. [Case Of](language/pattern-matching/README.md) — Value, range, and enum matching *(migrated)*
7. [Error Handling](language/error-handling/README.md) — `Result`, `Option`, `try`, `panic` *(migrated)*
8. [Concurrency](language/concurrency/README.md) — Tasks, task handles, and fork-join patterns *(migrated)*
9. [Units](program-structure/units.md) — Unit system, `uses`, namespaces *(migrated)*
10. [Projects](program-structure/projects.md) — `.fpasprj` project files, CLI, program/library kinds *(migrated)*
11. [Standard Library](11-stdlib.md) — `Std.*` built-in libraries
12. [Formatter style](fmt-style.md) — normative output rules for `fpas fmt`
13. [Formal grammar](../specs/grammar.ebnf) — ISO EBNF syntax annex (lexer + parser)

## Future Features

Features planned for later versions are documented in [docs/future/](../future/).
