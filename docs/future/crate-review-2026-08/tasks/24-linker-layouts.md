# Task 24 — Compare complete layout identity during linker coalescing

Status: open
Severity: P2
Difficulty: hard
Language gate: no
Depends on: none

## Goal

Private/public copies of the same record or enum coalesce only when their complete runtime/debugger
layout is equivalent. Incompatible copies fail with a deterministic link diagnostic.

## Verified cause

`fpas-linker/src/symbols.rs::matching_layout_definition` compares record field names and enum
variant/field names only. It ignores field types, record properties, and methods. Two incompatible
objects can therefore share one executable type ID.

## Critical constraint

`ObjectRecordLayout::field_types` and enum variant field types are indexes into each object's own
`debug_types` table. Equal numeric indexes across two objects are not proof of equal types, and
different indexes are not proof of different types. Do not implement direct vector equality.

## Required implementation

1. Build a cycle-safe structural debug-type comparison across two objects (primitive, aggregate
   name/identity, array/dictionary/task/function/procedure nesting as represented by object types).
2. Compare ordered field names plus structurally equivalent field types.
3. Compare record property names/getters and method names/routines under existing canonical-name
   rules.
4. Compare enum variants, ordered field names, and structural field types.
5. Preserve public-replaces-private/coalescing behavior only for fully equivalent layouts.

Reuse linker debug-type traversal in `src/debug_types.rs`; do not add a second generic type system.

## Tests

- Same field name with integer versus real type is rejected.
- Structurally equal types stored at different local debug-type indexes still coalesce.
- Nested and recursive matching layouts terminate and coalesce.
- Property/method mismatch is rejected; identical existing copies still coalesce.

## Verify

```text
cargo test -p fpas-linker
cargo build
cargo test --workspace
cargo fmt
```

## Done when

- Coalescing is based on complete structural identity, not local numeric IDs.
- Mismatches report both object/layout identities.
- Existing matching-copy behavior remains valid.
