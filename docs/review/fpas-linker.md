# `fpas-linker` review follow-up

Classification: linker correctness and executable validation. No language change expected.
Status: LINK-01 through LINK-03 completed; LINK-04 rejected with evidence on 2026-08-02.

| ID | Priority | Evidence | Finding and impact | Implementation direction | Required regression |
| --- | --- | --- | --- | --- | --- |
| LINK-01 | P1 | `crates/fpas-linker/src/lib.rs:97` | The linker checks only code/location parity, not `validate_executable`. Unknown intrinsics and invalid jump targets can be returned as a successful link. | Validate the completed chunk with the authoritative executable validator and expose a precise `LinkError`. | Unknown intrinsic, out-of-range and one-past-end jumps must fail `link_objects`. |
| LINK-02 | P1 | `crates/fpas-linker/src/lib.rs:188,216` | Unit terminal `Halt` is stripped after validation, but a function entry may point to that stripped instruction. Rebasing then points the function at the next object/program. | For non-root objects require every function entry to be strictly below the retained code length before registration. | Unit with only `Halt` and a function at offset zero; multi-object case proving no rebound. |
| LINK-03 | P2 | `crates/fpas-linker/src/lib.rs:141` | A public callable definition can resolve without a matching function-table implementation, producing an executable that fails name lookup later. | Track the defining object with each definition and require a case-insensitive matching function entry for callable definitions. | Missing function entry, wrong owner/name, case-insensitive valid match, and extra local functions. |
| LINK-04 | P3 | `crates/fpas-linker/src/lib.rs:188,224` | Validated relocation lists are rebuilt into `HashMap<u32, Vec<Relocation>>`, adding hashing and allocations. Impact is unmeasured. | Consume a peekable ordered relocation iterator while copying instructions. | Behavior-equivalence tests and a benchmark before claiming improvement. |

## Implementation notes

Resolve the validation boundary with BYTECODE-04/05 and UNIT-03. The test currently named `missing_private_and_kind_mismatched_imports_are_rejected` covers only private visibility; split it and add the missing kind-mismatch case.

## Implementation record

- LINK-01 routes the completed linked `Chunk` through `validate_executable` and preserves the typed
  `ExecutableError` in `LinkError::InvalidExecutable`. Unknown intrinsic identifiers and final
  out-of-range or one-past-end targets can no longer leave the linker successfully. Root-level
  `Return` remains a valid program exit; the full workspace example suite exposed and corrected
  the bytecode validator's initial rejection of that existing behavior.
- LINK-02 validates Unit function entries against the retained instruction count before appending
  any object. An entry at the removed terminal `Halt` is reported with its owner, name, offset, and
  retained code length, so it cannot silently rebind to the next object.
- LINK-03 records each definition together with its defining object and requires every callable
  definition to match a function-table name in that same object under ASCII case-insensitive
  lookup. Nested/local function-table entries without exported definitions remain valid.
- LINK-04 is rejected. The FPAS benchmark harness measures elapsed time inside already running
  programs and cannot isolate the one-time Rust linking path. No baseline can therefore support
  the proposed performance claim, and the existing relocation map remains unchanged. No benchmark
  history entry is recorded because no measured performance change was made.
- Definition/import validation and object appending were split into focused `definitions.rs` and
  `append.rs` modules. Link/rebase behavior remains in `tests/link.rs`; validation regressions now
  live in `tests/validation.rs` with one shared object fixture. The previously combined import test
  was split, and the missing kind-mismatch regression was added.
- Adjacent workspace Clippy failures were fixed without behavior changes in `fpas-language-service`,
  `fpas-cli`, and `fpas-lsp`; their existing targeted suites cover the refactors.
- `docs/pascal/program-structure/units.md` documents the final linker validation boundary. FPAS
  syntax, language semantics, and `Std.*` APIs are unchanged.

## Verification

- Baseline: `cargo test -p fpas-linker --locked` — passed: 10 tests plus doc tests.
- Targeted implementation: `cargo test -p fpas-linker --locked` — passed: 19 tests plus doc tests.
- Direct dependents and Clippy-adjacent crates: `cargo test -p fpas-linker
  -p fpas-language-service -p fpas-lsp -p fpas-compiler -p fpas-build --locked` — passed.
- Root-return integration regression: `cargo test -p fpas-cli --bin fpas
  main_tests::examples::example_fs_basics --locked` — passed.
- `cargo fmt --all -- --check` — passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — passed.
- `cargo build --workspace --locked` — passed.
- `cargo test --workspace --locked` — passed.
