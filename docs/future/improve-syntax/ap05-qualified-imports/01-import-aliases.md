# AP05.1: Import aliases

Package: [AP05: Qualified imports](README.md)

## Scope

Add `uses Unit as Alias;`. An aliased import opens no short names. Plain
imports keep their current behavior.

## Prerequisites

- AP01 (reference style), AP02 (diagnostic codes).

## Implementation

- Parser and AST: optional `as Alias` per imported unit in `uses`.
- Name resolution: an alias exposes the unit's public symbols only through
  `Alias.Name`; it adds no short names and no full-path access for that import.
- Diagnose alias collisions with another alias, a local name, or a unit
  root name, case-insensitively.
- Compiler and linker keep canonical unit identities; aliases are source-level
  only. Compiled-unit interfaces are unaffected.
- Formatter emits aliases.

## Affected areas

- `crates/fpas-parser/src/parser/program.rs`.
- `crates/fpas-sema/src/check/entry.rs`, `check/name_resolution/`,
  `interface/install.rs`.
- `crates/fpas-compiler` qualification of imported symbols; `fpas-fmt`.

## Migration

None required; plain imports are unchanged.

## Documentation

- `docs/specs/grammar.ebnf` (`uses_clause`).
- `docs/pascal/program-structure/units.md`.

## Verification

- Plain imports; alias-only access (short names and full paths rejected for
  that import); ambiguous short names; alias collisions with aliases and local
  names; aliased types, enum variants, constants, routines, and task calls.
- Importing a colliding helper cannot change a qualified call.
- Project test across compiled-unit reuse.
