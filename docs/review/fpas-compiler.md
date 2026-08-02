# `fpas-compiler` review follow-up

Classification: compiler correctness and refactoring. The listed fixes preserve documented FPAS semantics except for the newly specified enum exhaustion rule, which was explicitly agreed before implementation.
Status: COMPILER-01 through COMPILER-04 completed 2026-08-02.

| ID | Priority | Evidence | Finding and impact | Implementation direction | Required regression |
| --- | --- | --- | --- | --- | --- |
| COMPILER-01 | P1 | `crates/fpas-compiler/src/compiler/program/mod.rs:187-205` | **Done.** An explicit enum value of `i64::MAX` followed by an implicit member panicked in debug and wrapped to `i64::MIN` in release. Imported-interface and sema paths saturated instead. | Reject an implicit successor after `i64::MAX` consistently in semantic analysis, interface export, and lowering. | Program and compiled-unit cases at `i64::MAX`, including following implicit members and consistent debug/release behavior. |
| COMPILER-02 | P2 | `crates/fpas-compiler/src/unit_object.rs:156-280` | **Done.** Methods of a public record inherited the record's public flag even when the method/accessor was private. Linker metadata could therefore expose private callables. | Compute effective link visibility per routine and import only externally callable record members. | Assert definition flags and consumer imports for private/public methods, accessors, events, and static routines. |
| COMPILER-03 | P2 | `crates/fpas-compiler/src/compiler/designator/write.rs:43-56`, `src/compiler/expr/records.rs:43-64` | **Done.** Global index-chain depth was truncated to `u8`; record field/update counts were truncated to `u16`. The pushed values no longer matched bytecode operands. | Route all operand widths through existing checked conversion helpers before emission. | Boundary AST tests for 255/256 indices and 65,535/65,536 fields. |
| COMPILER-04 | P3 | `crates/fpas-compiler/src/lib.rs:47-78`, `src/unit_object.rs:42-82,139-170`, `src/compiler/mod.rs:121-160` | **Done.** Twelve sema metadata maps were manually destructured and threaded through a twelve-argument constructor, making miswiring easy. | Introduce the named analysis metadata structure planned with `fpas-sema` and pass it as one coherent context. | API/compile tests ensure every metadata map reaches lowering; no behavior change. |

## Implementation notes

Implement COMPILER-04 together with SEMA-03, not as two temporary adapters. Existing compiler tests are broad; the missing value-width and emitted-visibility assertions should be added before production changes.

Documentation normally remains unchanged for internal correctness fixes. If COMPILER-01 requires choosing a previously unspecified language rule, stop and request explicit agreement before code or normative docs change.

## Implementation record

- COMPILER-01 now allows `i64::MAX` as an explicit final enum backing value and reports F2018 when
  a later member requires an implicit successor. A later explicit value restarts the sequence.
  Sema, persisted unit-interface export, imported-interface registration, and local lowering all
  use checked progression. The agreed rule is documented in `docs/pascal/language/types/enums.md`.
- COMPILER-02 now marks an object definition public only when its record type and effective member
  are public. Private routines remain local unless they back a public property or event that must
  invoke them across the unit boundary. Consumer objects no longer import unrelated private
  routines. The regression covers instance functions/procedures, property accessors, and static
  functions/procedures with private and public visibility.
- COMPILER-03 now validates global index-chain depths and record literal/update field counts
  before emitting values or their width-limited bytecode operands. Focused regressions cover the
  accepted maximum conversion and the first rejected AST count, including raw literals,
  default-expanded literals, and record updates.
- COMPILER-04 now passes one named `AnalysisMetadata` value from every program and unit-object
  analysis entry point into `Compiler::new`. The constructor destructures named fields once instead
  of accepting twelve positional maps.
- A compiler regression supplies a distinct nonempty sentinel in every lowering metadata map and
  verifies that each one reaches the matching compiler field.
- COMPILER-04 itself leaves compiler behavior and user-facing documentation unchanged; its related
  sema API contract is recorded in `fpas-sema.md`.

## Verification

- `cargo test -p fpas-compiler --locked compiler::designator::write::tests` — passed: 2 tests.
- `cargo test -p fpas-compiler --locked compiler::expr::records::tests` — passed: 5 tests.
- `cargo test -p fpas-compiler --locked --test object_visibility` — passed: 2 integration tests.
- `cargo test -p fpas-sema --locked enum_` — passed: 27 targeted tests.
- `cargo test -p fpas-diagnostics --locked` — passed: 17 tests plus doc tests.
- `cargo test -p fpas-compiler --release --locked tests::enums::errors::enum_implicit_backing_value_after_i64_max_is_rejected -- --exact` — passed: 1 release-mode regression.
- `cargo test -p fpas-compiler --locked` — passed: 982 unit, 2 object-visibility,
  14 record-visibility, and 11 unit-object tests plus doc tests.
- `cargo fmt --all -- --check` — passed.
- `cargo build --workspace --locked` — passed.
- `cargo test --workspace --locked --quiet` — passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — passed.
