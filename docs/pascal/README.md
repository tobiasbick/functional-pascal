# Functional Pascal

A function-first programming language built on Pascal's readable syntax and clean structure. Runs on a managed virtual machine — no pointers, no manual memory management.

## Design Principles

- **Function First** — Functions are the primary abstraction. No classical classes or object hierarchies.
- **Immutable by Default** — Variables require `mutable var` block to allow reassignment.
- **Safe by Design** — No pointers, no unsafe operations. The VM handles memory.
- **Explicit Types** — Every binding explicitly states its type.
- **Case-Insensitive** — Keywords and identifiers are case-insensitive, as in classical Pascal.
- **Familiar Syntax** — Pascal's `begin`, `end`, `:=`, `downto`, and other well-known constructs.

## Table of Contents

1. [Overview](01-overview.md) — Philosophy, hello world, first taste
2. [Basics](02-basics.md) — Primitive types, variables, constants, operators
3. [Control Flow](03-control-flow.md) — Conditionals, loops, branching
4. [Functions](04-functions.md) — Functions, procedures, first-class functions, nested functions
5. [Types](05-types.md) — Records, enumerations, arrays, type aliases
6. [Case Of](06-pattern-matching.md) — Value, range, and enum matching
7. [Error Handling](07-error-handling.md) — `Result`, `Option`, `try`, `panic`
8. [Concurrency](08-concurrency.md) — Planned for future versions
9. [Units](09-units.md) — Unit system, `uses`, namespaces, visibility
10. [Projects](10-projects.md) — `.fpasprj` project files, CLI, program/library kinds
11. [Standard Library](11-stdlib.md) — `Std.*` built-in libraries

## Future Features

Features planned for later versions are documented in [docs/future/](../future/).
