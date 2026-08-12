# Verification matrix

Status values are `PASS`, `PENDING`, or `NOT RUN`.

| ID | Acceptance case | Evidence | Status |
| --- | --- | --- | --- |
| VR-T01 | Fieldless and data-carrying enum constructors evaluate | `fpas-vm` variant replacement tests | PASS |
| VR-T02 | Constructor arguments share the expression operation budget | `constructor_arguments_evaluate_once_under_the_shared_operation_budget` | PASS |
| VR-T03 | Wrong constructor field type fails as `EvaluationType` | `constructor_and_replacement_failures_are_atomic` | PASS |
| VR-T04 | Multi-segment fieldless constructor is invoked once with its full name | `evaluation_resolves_a_qualified_fieldless_constructor_once` | PASS |
| VR-T05 | Complete enum, `Result`, and `Option` roots commit and continue | `fpas-vm` variant replacement tests | PASS |
| VR-T06 | Nested roots, globals, arrays, dictionaries, parameters, and captures follow existing mutation rules | VM and JSONL variant replacement tests | PASS |
| VR-T07 | Rejected VM operations preserve values and handles | `constructor_and_replacement_failures_are_atomic` | PASS |
| VR-T08 | JSONL reports wrong constructor field types as `evaluation_type` | `crates/fpas-debug/tests/variant_replacement.rs` | PASS |
| VR-T09 | DAP rejects wrong constructor field types without invalidation | `crates/fpas-debug/tests/dap_variant_replacement.rs` | PASS |
| VR-T10 | VS Code maps standard mutation requests and refreshes variables | `editors/vscode/test/debugger_host/variant_replacement.ts` | PASS |
| VR-T11 | FPAS fixture formatting is stable | `fpas fmt --check tests/debugger/fixtures/variant_replacement.fpas` | PASS |
| VR-T12 | Rust formatting is stable | `cargo fmt --all -- --check` | PASS |
| VR-T13 | Workspace builds | `cargo build` | PASS |
| VR-T14 | Full workspace regression suite passes | `cargo test --workspace --no-fail-fast` | PASS |

Focused evidence already run after the corrective review:

- `cargo test -p fpas-vm evaluation_resolves_a_qualified_fieldless_constructor_once`
- `cargo test -p fpas-vm variant_replacement`
- `cargo test -p fpas-debug --test variant_replacement`
- `cargo test -p fpas-debug --test dap_variant_replacement`
- `(cd editors/vscode && npm test)`
- `cargo run -p fpas-cli -- fmt --check tests/debugger/fixtures/variant_replacement.fpas`
- `cargo fmt --all -- --check`
- `cargo build`
- `cargo test --workspace --no-fail-fast`
