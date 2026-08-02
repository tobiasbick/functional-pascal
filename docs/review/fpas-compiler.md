# `fpas-compiler` review follow-up

Classification: compiler correctness and refactoring. The listed fixes preserve documented FPAS semantics; do not invent a new overflow rule without checking the language docs and obtaining agreement if necessary.
Status: COMPILER-04 completed 2026-08-02; COMPILER-01 through COMPILER-03 remain open.

| ID | Priority | Evidence | Finding and impact | Implementation direction | Required regression |
| --- | --- | --- | --- | --- | --- |
| COMPILER-01 | P1 | `crates/fpas-compiler/src/compiler/program/mod.rs:187-205` | An explicit enum value of `i64::MAX` followed by an implicit member panics in debug and wraps to `i64::MIN` in release. Imported-interface and sema paths currently saturate instead. | First confirm the documented rule. Then use the same checked or saturating policy in every compilation/interface path and emit a diagnostic if the spec rejects exhaustion. | Program and compiled-unit cases at `i64::MAX`, including following implicit members and consistent debug/release behavior. |
| COMPILER-02 | P2 | `crates/fpas-compiler/src/unit_object.rs:190-220` | Methods of a public record inherit the record's public flag even when the method/accessor is private. Linker metadata can therefore expose private callables. | Compute visibility per routine: the type and the member must both be public. | Assert emitted object definition flags for private/public methods, accessors, and static routines. |
| COMPILER-03 | P2 | `crates/fpas-compiler/src/compiler/designator/write.rs:43-56`, `src/compiler/expr/records.rs:43-64` | Global index-chain depth is truncated to `u8`; record field/update counts are truncated to `u16`. The pushed values no longer match bytecode operands. | Route all operand widths through existing checked conversion helpers before emission. | Boundary AST tests for 255/256 indices and 65,535/65,536 fields. |
| COMPILER-04 | P3 | `crates/fpas-compiler/src/lib.rs:47-78`, `src/unit_object.rs:42-82,139-170`, `src/compiler/mod.rs:121-160` | **Done.** Twelve sema metadata maps were manually destructured and threaded through a twelve-argument constructor, making miswiring easy. | Introduce the named analysis metadata structure planned with `fpas-sema` and pass it as one coherent context. | API/compile tests ensure every metadata map reaches lowering; no behavior change. |

## Implementation notes

Implement COMPILER-04 together with SEMA-03, not as two temporary adapters. Existing compiler tests are broad; the missing value-width and emitted-visibility assertions should be added before production changes.

Documentation normally remains unchanged for internal correctness fixes. If COMPILER-01 requires choosing a previously unspecified language rule, stop and request explicit agreement before code or normative docs change.

## Implementation record

- COMPILER-04 now passes one named `AnalysisMetadata` value from every program and unit-object
  analysis entry point into `Compiler::new`. The constructor destructures named fields once instead
  of accepting twelve positional maps.
- A compiler regression supplies a distinct nonempty sentinel in every lowering metadata map and
  verifies that each one reaches the matching compiler field.
- Compiler behavior and user-facing compiler documentation are unchanged; the related sema API
  contract is recorded in `fpas-sema.md`.

## Verification

- `cargo test -p fpas-compiler --locked` — passed: 974 unit, 14 record-visibility, and 9
  unit-object tests plus doc tests.
- `cargo fmt --all -- --check` — passed.
- `cargo build --workspace --locked` — passed.
- `cargo test --workspace --locked --quiet` — passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — passed.
