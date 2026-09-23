# Structure, duplication, and defensive code

## S1. Publication code has diverged across artifact types

Priority P3, with the P2 correctness consequence described in C2.

Compare these implementations:

- `crates/fpas-unit/src/sidecar/atomic.rs` implements custom temporary names, backup/restore publication on Windows, cleanup, and lock retry loops.
- `crates/fpas-build/src/program_artifact/atomic.rs` uses `AtomicWriteFile`, an OS lock, and typed pending commit ownership.
- `crates/fpas-bundle/src/publication.rs` uses `AtomicWriteFile` and a commit-injection seam for testing.

These are not identical transactions: units require shared reader coordination, program builds retain a lock over more work, and bundles set executable permissions. Preserve those differences. The duplicated responsibility worth consolidating is complete-file replacement and failed-staging cleanup. Prefer reusing the existing library mechanism over introducing a new generic artifact framework.

A useful acceptance test is the bundle test's existing assertion that a failed commit preserves the previous destination without a backup/restore phase. Bring equivalent coverage to units.

## S2. Record literal lowering repeats staging and emission

Priority P3. `crates/fpas-compiler/src/lowering/aggregates/records.rs:15` and line 76 implement `lower_record_literal` and `lower_record_literal_as`. Both collect provided fields, resolve defaults, lower and stage values into hidden locals, read the staged values, and emit `Operation::MakeRecord`.

The type-resolution entry points differ and should remain understandable. The repeated staging/emission sequence risks fixes being applied to only one record-construction path. Extract a narrow internal operation after field expressions and expected types have been resolved. Preserve evaluation order and default-expression behavior. This recommendation does not establish a current evaluation-order bug.

Cover both inferred and context-typed literals, default fields, and side-effectful initializers with the same expected ordering. Existing aggregate tests provide a baseline; add paired cases if the two entry points are not both exercised.

## S3. Some defensive code obscures invariants

Priority P3. Three specific examples merit cleanup when these modules are next changed:

1. `crates/fpas-program/src/format/sections.rs:91` requires `count == TAGS.len()`, currently 11, then checks `count > MAX_SECTIONS`, currently 64. The second runtime branch cannot be reached in the current format. Express the relationship between the static tag table and the format maximum as a static assertion or invariant test.
2. In the same file, range validation rejects an absent or out-of-bounds `end`, then `end.unwrap_or(payload.len())` supplies a fallback that should be impossible. Bind the validated end directly with pattern matching. The later bounded slice lookup is appropriate at a decoding boundary; it should not be removed indiscriminately.
3. `crates/fpas-linker/src/plan/layouts.rs:140` and line 155 replace a failed layout-index conversion with `u32::MAX`. A sentinel silently substitutes a different target identity. If prior object validation establishes the index range, express that invariant; otherwise propagate a conversion error. Do not silently proceed with a fabricated ID.

The linker also retains an unused `definition_count` method under `allow(dead_code)` at `crates/fpas-linker/src/plan/symbols.rs:172`, justified by possible diagnostics. There is no current caller. Remove it unless an actual caller is introduced by the relevant work.

These are maintenance findings. No measured slowdown or reachable user-input miscompilation is claimed for the redundant checks or sentinel conversion.

## Defenses that should remain

The project/source hash checks in `fpas-build/src/source_snapshot.rs` prevent compiling or publishing against a graph whose source changed. Bounded decoding, bytecode verification, network cancellation checks, and artifact locks establish different guarantees. They should not be called excessive merely because they repeat some information available elsewhere.

Likewise, Rust test assertions using `unwrap` or `expect` are not production error-handling defects. Poison recovery and saturating arithmetic need a concrete invalid state or meaningful hot-path cost before they become review findings. No blanket removal is proposed.

## File layout

Production code is already divided into many concern-specific directories. The largest Rust files found were mostly tests. Production exceptions above 500 lines included `crates/fpas-bytecode/src/validate/instruction/abc.rs` at 531 lines and `crates/fpas-bytecode/src/instruction.rs` at 522 lines. The instruction schema is cohesive; its size alone is insufficient reason to split it.

When extending ABC validation, consider opcode-family files inside `validate/instruction/abc/`. Large test files such as `fpas-bytecode/tests/bytecode/verifier.rs` and `fpas-debug/tests/dap.rs` can be grouped by behavior when next edited. Avoid an unrelated workspace-wide reorganization as a prerequisite for the concrete fixes above.
