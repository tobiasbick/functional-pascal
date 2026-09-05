---
name: rust-refactor-helper
description: >
  Refactor Rust symbols and modules in this repository while preserving behavior.
  Use for symbol renames, function extraction or inlining, and moves between modules.
---

# Rust refactoring

Follow [AGENTS.md](../../../AGENTS.md) for module layout, scope, and verification.
Use [rust-best-practices](../rust-best-practices/SKILL.md) for ownership and error
handling, and [fpas-change-checklist](../fpas-change-checklist/SKILL.md) to check
documentation and tests.

Before editing, find the definition and its callers with scoped `rg` searches.
Inspect imports, re-exports, trait implementations, macros, tests, and documentation;
a text match alone does not establish symbol identity. If semantic navigation is
available in the current session, use its documented operations to supplement the
search. Report any unresolved references rather than claiming complete coverage.

Show the affected paths and intended layout before applying the refactor. A
preview-only request ends with that analysis; an implementation request authorizes
proceeding within the requested scope.

- **Rename:** check for name conflicts, then update the definition, references,
  imports, re-exports, and documentation that names the symbol.
- **Extract or inline:** preserve ownership, lifetimes, evaluation order, early
  returns, error propagation, and loop control. Choose parameters and results from
  the actual data flow.
- **Move:** inspect module dependencies and visibility, update `mod` declarations
  and imports, and remove empty modules or dead re-exports exposed by the move.

Run `cargo fmt`, `cargo build`, and `cargo test --workspace`, plus relevant FPAS
checks from the change checklist. Search for stale symbol names and paths, inspect
the final diff, and report the verification results and any remaining uncertainty.
A refactor must preserve FPAS language behavior; changing it requires explicit
user agreement.
