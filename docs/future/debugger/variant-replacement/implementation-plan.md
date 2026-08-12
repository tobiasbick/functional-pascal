# Implementation plan

## Work packages

| ID | Work package | Main ownership | Status |
| --- | --- | --- | --- |
| VR-01 | Parse and lower explicit constructor expressions | `crates/fpas-debug/src/evaluation/` | Complete |
| VR-02 | Resolve exact enum constructors from runtime layouts | `crates/fpas-vm/src/vm/debug/calls/resolution.rs` | Complete |
| VR-03 | Construct detached enum values with arity and recursive field-type validation | `crates/fpas-vm/src/vm/debug/calls/enum_constructor.rs` | Complete |
| VR-04 | Resolve multi-segment fieldless constructor names without prefix calls | `crates/fpas-vm/src/vm/debug/evaluation/` | Complete |
| VR-05 | Commit complete enum, `Result`, and `Option` values through existing atomic mutation | `crates/fpas-vm/src/vm/debug/mutation/` | Complete |
| VR-06 | Map JSONL, DAP, and VS Code surfaces without custom capability claims | `crates/fpas-debug/`, `editors/vscode/` | Complete |
| VR-07 | Synchronize current user documentation and deferred boundaries | `docs/pascal/tools/`, `docs/future/debugger/` | Complete |
| VR-08 | Run focused and full verification gates | workspace | Complete |

## Dependency order

`VR-01 -> VR-02 -> VR-03 -> VR-04 -> VR-05 -> VR-06 -> VR-07 -> VR-08`

## Exit gates

- Positive replacement covers fieldless and data-carrying enum variants plus
  both branches of `Result` and `Option`.
- Negative coverage rejects unknown, ambiguous, wrong-arity, and wrong-field-
  type constructors before mutation.
- Fully qualified fieldless names with more than two segments resolve exactly
  once.
- JSONL and DAP failures preserve the old value; DAP emits no invalidation.
- VS Code host tests prove standard DAP mapping and refresh behavior.
- `cargo fmt --all -- --check`, `cargo build`, and
  `cargo test --workspace --no-fail-fast` pass.
