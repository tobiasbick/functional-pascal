# Verification matrix

Status values: `NOT_RUN`, `PASS`, `FAIL`, `BLOCKED`.

Evidence must name the exact test or command. A package may not be marked
complete from an informal manual observation.

| ID | Level | Requirement | Planned evidence | Status | Evidence |
|---|---|---|---|---|---|
| AGC-T01 | VM unit | Discovery returns canonical fieldless, single-field, and multi-field enum descriptors in metadata order | `variant_construction::discovery` tests | PASS | `cargo test -p fpas-vm --lib variant`; `discovery_returns_canonical_enum_and_wrapper_descriptors` |
| AGC-T02 | VM unit | Discovery returns canonical `Ok`, `Error`, `Some`, and `None` descriptors | `variant_construction::wrapper_discovery` tests | PASS | same test covers `Outcome`/`Optional`; JSONL `jsonl_variant_describe_is_read_only_and_canonical` |
| AGC-T03 | VM unit | Fieldless enum and `Option.None` construction commits a complete value | construction positive tests | PASS | `fieldless_and_wrapper_construction_commits_complete_values` |
| AGC-T04 | VM unit | Multi-field and single-field enum construction evaluates fields in declaration order | ordered-evaluation tests with observable detached-call counters | PASS | `multi_field_construction_evaluates_declaration_order`; JSONL Pair `Left=1` `Right=2` after `{"Right":"Next()","Left":"Next()"}` |
| AGC-T05 | VM unit | `Result` and `Option` construction validates exact payload types | wrapper construction positive and type-negative tests | PASS | `fieldless_and_wrapper_construction_commits_complete_values` (`None`/`Some`/`Error`/`Ok`); `construction_failures_preserve_the_original_value` type mismatch |
| AGC-T06 | VM unit | Nested writable targets and uninitialized mutable roots work; uninitialized descendants reject | storage-boundary tests | PASS | `nested_and_uninitialized_targets_construct_while_descendants_reject`; JSONL `WrappedChoice.value`, `WrappedChoices[0].value`, and `OuterValue.Item` |
| AGC-T07 | VM unit | Unknown variant and missing, extra, unknown, or ASCII-case-duplicate fields reject before evaluation | exact-field negative tests | PASS | `exact_field_set_rejects_before_evaluation`; JSONL `variant_unknown` / `variant_field_set` |
| AGC-T08 | VM unit | Parse, evaluation, forbidden call, limit, type, metadata, mutability, cancellation, and expiry failures preserve the original live value | atomic rollback parameterized tests | PASS | `construction_failures_preserve_the_original_value`; `discovery_clears_a_pending_evaluation_cancellation`; expired frame `UnknownFrame` after commit; JSONL parse offset plus unchanged `Selected` |
| AGC-T09 | VM unit | Existing complete replacement and qualified single-payload transition behavior is unchanged | existing `variant_replacement` and `variant_transition` suites | PASS | `cargo test -p fpas-vm --lib variant` (23 passed); `cargo test -p fpas-debug --test variant_replacement --test variant_transition --test dap_variant_replacement --test dap_variant_transition` |
| AGC-T10 | JSONL | `variant.describe` is deterministic, read-only, stopped-only, and LLM-readable | `crates/fpas-debug/tests/variant_construction.rs` | PASS | `jsonl_variant_describe_is_read_only_and_canonical`; running-state `invalid_state`; non-wrapper `variable_path_unsupported` |
| AGC-T11 | JSONL | `variant.construct` supports all shapes and returns canonical variant plus standard value summary | JSONL positive contract tests | PASS | `jsonl_variant_construct_commits_and_continues` (`Choice.Empty`, `choice.pair`, nested `Count`, `Ok`, dictionary `Count`) |
| AGC-T12 | JSONL | Malformed arguments, expressions, names, fields, and state return stable error objects and do not mutate | JSONL negative/edge contract tests | PASS | `jsonl_variant_construct_rejects_without_mutation` (parse offset, unknown, missing/extra/duplicate fields, omitted `fields`, evaluate unchanged) |
| AGC-T13 | DAP | Describe and construct custom requests map frame, target, variant, and fields exactly | `crates/fpas-debug/tests/dap_variant_construction.rs` | PASS | `dap_variant_describe_and_construct_map_jsonl_and_invalidate_variables` |
| AGC-T14 | DAP | Successful construct emits negotiated variables invalidation after its response; failure and discovery do not | DAP ordering tests with both client capabilities | PASS | same test (`described.len()==1`, `constructed.len()==2`); `dap_variant_construct_omits_invalidation_on_failure_and_without_capability` |
| AGC-T15 | VS Code | Command contribution, registration, discovery Quick Pick, declaration-order prompts, cancellation, and forwarding work | command unit/contract tests | PASS | `editors/vscode/scripts/verify-manifest.mjs` command `functionalPascal.debug.constructVariant`; host registration assertion; Extension Host rejects an unexpected programmatic `Extra` field without invalidation |
| AGC-T16 | VS Code host | Real Extension Host constructs fieldless and multi-field variants, observes Variables refresh, continues, and verifies program output | `editors/vscode/test/debugger_host/variant_construction.ts` | PASS | `npm test --prefix editors/vscode`; `verifyVariantConstruction` output `3\n` |
| AGC-T17 | Regression | Existing breakpoints, evaluation, mutation, variant replacement/transition, uninitialized assignment, and function assignment stay green | targeted Rust and Extension Host suites | PASS | VM/JSONL/DAP variant suites plus full `npm test` debugger host sequence |
| AGC-T18 | Quality | Rust and TypeScript formatting/type checks pass | `cargo fmt --all -- --check`; `npm test` | PASS | both exit 0 |
| AGC-T19 | Repository | Full Rust build and tests pass | `cargo build`; `cargo test --workspace --no-fail-fast` | BLOCKED | `cargo build` exit 0; workspace sole failure is `repository_references_find_notes_update_in_the_consuming_program` (23 vs 22 `NotesUpdate` refs), unrelated to this package |
| AGC-T20 | Packaging/docs | VSIX packaging contracts, current debugger docs, plan status, and whitespace checks agree | `npm run package`; `git diff --check`; documentation review | PASS | packaged `functional-pascal-0.3.0-win32-x64.vsix` with `variantConstructionCommand.js`; `git diff --check` CRLF conversion warnings only |

## Traceability

| Work package | Required rows |
|---|---|
| AGC-01 | AGC-T01, AGC-T02, AGC-T09 |
| AGC-02 | AGC-T01, AGC-T02, AGC-T06, AGC-T10 |
| AGC-03 | AGC-T03 through AGC-T09 |
| AGC-04 | AGC-T08, AGC-T17, AGC-T18 |
| AGC-05 | AGC-T10 through AGC-T12 |
| AGC-06 | AGC-T13, AGC-T14 |
| AGC-07 | AGC-T15, AGC-T16 |
| AGC-08 | AGC-T17 through AGC-T20 |
