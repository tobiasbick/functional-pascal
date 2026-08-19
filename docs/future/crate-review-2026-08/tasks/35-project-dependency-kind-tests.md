# Task 35 — Cover rejection of non-library dependencies

Status: coverage
Severity: P3 coverage gap
Difficulty: easy
Language gate: no
Depends on: none

## Existing behavior

`fpas-project/src/dependencies.rs::ensure_library_dependency` already rejects any dependency whose
project kind is not `library`, matching
[`Projects`](../../../pascal/program-structure/projects.md#dependencies-section).

## Goal

Lock that rule down for both dependency syntaxes without changing production behavior.

## Tests

- `[dependencies].projects` pointing at a `kind = "program"` manifest returns the existing
  actionable error.
- `[dependencies].workspace` naming a program member returns the same policy error.
- A library dependency still loads.

Use existing temporary project/workspace fixtures; do not add a second manifest builder.

## Verify

```text
cargo test -p fpas-project
cargo build
cargo test --workspace
cargo fmt
```

## Done when

- Both dependency forms have direct regression coverage.
- Production code and docs remain unchanged unless the test exposes a real inconsistency.
