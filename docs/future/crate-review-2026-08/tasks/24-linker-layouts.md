# Task 24 — Linker coalescing must include field types

Status: open
Severity: P2
Difficulty: medium
Language gate: no
Depends on: none

## Goal

Two record (or enum) definitions that share a name and field **names** but not field **types** must not be merged into one layout. First-wins must not drop a public layout in favor of an earlier private copy if that is already specified — fix types first.

## Bug

`crates/fpas-linker/src/symbols.rs`: duplicate record/enum copies coalesce when field names match (case-insensitive). `field_types`, properties, and methods are ignored. Linker tests already construct objects (`matching private layout copies must coalesce`). The same fixture with `x: integer` vs `x: real` would link and share one layout.

## Fix

Treat layouts as equal only when names **and** types (and, if cheap, property/method names) match. If they clash, return a link error with both object identities. Do not silently pick the first.

If public-replaces-private is existing behavior, keep it **only** when types match.

## Tests

Extend the existing coalesce test: `x: integer` vs `x: real` → link error. Matching copies still coalesce.

## Verify

```text
cargo test -p fpas-linker
cargo fmt
```

## Done when

- Type-mismatched duplicates fail link.
- Identical copies still coalesce.
- Docs unchanged.
