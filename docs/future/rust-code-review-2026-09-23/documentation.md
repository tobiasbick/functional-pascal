# Documentation findings

## D1. Atomic unit-sidecar publication is overstated

Priority P2. `docs/pascal/program-structure/units.md:142` says sidecars are written atomically. `crates/fpas-unit/src/sidecar/mod.rs:141` makes the same claim for `write_sidecar`. The Windows implementation performs backup and replacement renames separately, as described in C2.

Preferred follow-up: make the implementation meet the documented guarantee and add failure-path tests. If the weaker behavior is intentionally retained, the documentation must distinguish coordinated readers from persistence across process failure. This report does not change the documented language or build contract.

## D2. Borrowed intrinsic arguments still copy read-only text

Priority P3. The comment at `crates/fpas-std/src/intrinsic_args.rs:12` says implementations clone only returned or mutated values. `pop_string` at line 84 allocates an owned copy, including for read-only parsing in `parse.rs` and `conv.rs`.

Either complete the borrowing migration for those call sites or narrow the comment to the actual guarantee: argument windows are borrowed, while some decoders still materialize owned values. See P4 for the implementation opportunity. Do not document a completed allocation optimization until it exists.

## D3. Implemented codec limits are described as future work

Priority P3. `crates/fpas-bytecode/src/limits.rs:41` onward describes `MAX_LINKED_UNITS`, `MAX_IDENTITY_STRING_BYTES`, `MAX_SECTIONS`, and `MAX_PAYLOAD_BYTES` as reserved for a later persistent executable codec.

The codec already exists in `crates/fpas-program/src/format/`, including header decoding, the section directory, and bounded payload decoding. Replace these comments with the current resource they limit. This is documentation-only work and requires no behavior change.

## Checks without findings

A repository-wide scan of explicit `docs/pascal/*.md` references in Rust files found no missing target files. This only checks path existence; it does not validate every Markdown fragment, semantic statement, or rendered rustdoc link.

The `Std.Path` page correctly states that `BaseName` follows host Rust path parsing. C3 is a bad platform-specific test expectation, not a reason to change that public documentation.

Current user-facing documentation was left unchanged. All proposed work and findings are contained in this review directory.
