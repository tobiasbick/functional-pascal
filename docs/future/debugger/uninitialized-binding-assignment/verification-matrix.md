# Verification matrix

| Requirement | Regression or evidence | State |
|---|---|---|
| UBA-S01 local handle initialization | `mutable_local_and_global_roots_initialize_from_handles_and_names` | implemented |
| UBA-S02 global handle initialization from `None` | same VM session test asserts `<uninitialized>` before `set_variable` | implemented |
| UBA-S03 textual local/global initialization | VM session test and JSONL/DAP transcripts | implemented |
| UBA-S04 JSONL mapping | `jsonl_uninitialized_roots_initialize_atomically_and_continue` | implemented |
| UBA-S04 DAP mapping and invalidation | `dap_set_variable_and_set_expression_initialize_uninitialized_roots` | implemented |
| UBA-S04 VS Code forwarding | `verifyUninitializedAssignment` Extension Host scenario | implemented |
| UBA-S05 selected child task | JSONL child-task test asserts `<uninitialized>` before assignment | implemented |
| UBA-S05 failures are atomic | `replacement_evaluates_once_and_failures_preserve_empty_storage` | implemented |
| UBA-S05 stale handles | session, JSONL, and DAP mutation regressions | implemented |
| UBA-S06 initialized `unit` | `initialized_unit_is_not_rendered_as_the_empty_sentinel` | implemented |
| Register write/take lifecycle | `writes_takes_and_unit_stores_update_initialization_bits` | implemented |
| Calls, callbacks, intrinsics, tasks, frame reuse | `register_initialization` and `register_stack` VM tests | implemented |
| FPAS fixture formatting | `fpas fmt --check tests/debugger/fixtures/uninitialized_assignment.fpas` | verified 2026-08-12 |
| Rust formatting | `cargo fmt` and final `cargo fmt --check` | verified 2026-08-12 |
| Workspace build | `cargo build` | verified 2026-08-12 |
| Workspace tests | `cargo test --workspace --no-fail-fast` | verified 2026-08-12 |
| VS Code Extension Host | `npm test` from `editors/vscode` | verified 2026-08-12 |
