# Debugger integration tests

Five explicit Cargo targets share the 73 case modules in this directory.
`autotests = false` in the crate manifest prevents each case file from also
becoming an executable. Add a new case module to exactly one suite entry point.

| Target | Case modules | Protected boundary |
| --- | --- | --- |
| [dap_protocol](suites/dap_protocol.rs) | 32 | DAP framing, request mapping, event ordering and capabilities |
| [jsonl_protocol](suites/jsonl_protocol.rs) | 4 | JSONL transport, launch ownership and protocol transcripts |
| [execution_control](suites/execution_control.rs) | 15 | Breakpoints, stopped state, task control, reload and replay |
| [storage_mutation](suites/storage_mutation.rs) | 16 | Atomic mutation, live storage ownership and expired inspection handles |
| [evaluation](suites/evaluation.rs) | 6 | Expression parsing, detached calls, parameter modes and portable metadata |

Explicit support paths in `dap.rs` and `transport.rs` keep their shared reader
fixtures available when the files are loaded as modules.
Storage-mutation cases share one suite-owned parameter-copy stopping helper.

## Case selection

Select a suite target and filter by `module::case`. Nested modules add their own
prefix: a case in `call_evaluation::named_arguments` uses
`call_evaluation::named_arguments::case` in the `evaluation` target.

```sh
cargo test --locked -p fpas-debug
cargo test --locked -p fpas-debug --test jsonl_protocol
cargo test --locked -p fpas-debug --test storage_mutation -- variable_mutation::jsonl_sets_variables_with_stable_errors_and_fresh_handles --exact
```

## Case modules

Each listed module loads the corresponding `<name>.rs` case file.
Cases own their debugger sessions and scripted input within the suite process.
They do not mutate the environment, install a global allocator or re-execute the
test binary. The call-evaluation request counter uses atomic increments and does
not require a fresh process.

### dap_protocol

- `dap`
- `dap_attach`
- `dap_breakpoint_assign`
- `dap_capturing_routine_assignment`
- `dap_cell_capturing_routine_assignment`
- `dap_completed_result`
- `dap_data_breakpoints`
- `dap_dictionary_mutation`
- `dap_empty_storage_construction`
- `dap_expression_mutation`
- `dap_forced_return`
- `dap_frame_restart`
- `dap_function_breakpoints`
- `dap_function_value_assignment`
- `dap_hot_reload`
- `dap_instruction_change`
- `dap_location`
- `dap_payload_mutation`
- `dap_quiescence`
- `dap_record_replay`
- `dap_runtime_failure_filters`
- `dap_sequence_mutation`
- `dap_task_control`
- `dap_task_debugging`
- `dap_task_handle_assignment`
- `dap_task_lifecycle`
- `dap_transport`
- `dap_uninitialized_assignment`
- `dap_variable_mutation`
- `dap_variant_construction`
- `dap_variant_replacement`
- `dap_variant_transition`

### jsonl_protocol

- `attach`
- `jsonl`
- `jsonl_contract`
- `transport`

### execution_control

- `breakpoint_assign`
- `completed_result`
- `data_breakpoints`
- `forced_return`
- `frame_restart`
- `function_breakpoints`
- `hot_reload`
- `instruction_change`
- `location`
- `quiescence`
- `record_replay`
- `runtime_failure_filters`
- `task_control`
- `task_debugging`
- `task_lifecycle`

### storage_mutation

- `capturing_routine_assignment`
- `cell_capturing_routine_assignment`
- `dictionary_mutation`
- `empty_storage_construction`
- `expression_mutation`
- `function_value_assignment`
- `payload_mutation`
- `sequence_mutation`
- `sequence_task_mutation`
- `session_sequence_mutation`
- `task_handle_assignment`
- `uninitialized_assignment`
- `variable_mutation`
- `variant_construction`
- `variant_replacement`
- `variant_transition`

### evaluation

- `bits_evaluation`
- `boolean_evaluation`
- `call_evaluation`
- `logical_precedence`
- `unicode_character_literals`
- `var_parameters`
