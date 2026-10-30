# Language-service integration tests

Four explicit Cargo targets load 31 case modules.
The crate uses `autotests = false`; add each new case module to exactly one
suite.

| Target | Case modules | Protected boundary |
| --- | --- | --- |
| [analysis_state](suites/analysis_state.rs) | 11 | Incremental diagnostics, document versions, checked types and public snapshots |
| [project_context](suites/project_context.rs) | 3 | Project indexes, repository discovery and workspace ownership |
| [editor_queries](suites/editor_queries.rs) | 8 | Completion, hover, API metadata, call signatures and constructor queries |
| [navigation_edits](suites/navigation_edits.rs) | 9 | Canonical symbol identities, references, rename, semantic tokens and refactorings |

## Fixtures and isolation

Each suite declares the shared `support` module once. Case modules import that
module through `crate::support`, so their temporary directories share one atomic
counter per executable. The counter, process ID and descriptive label separate
fixture paths even when cases run concurrently with the same label. Each
`TempDirectory` owns its files and removes its directory on drop.

The tests own their service/document state and do not mutate process environment
variables, install a global allocator or re-execute the test binary. No case
requires a standalone process. Tests that inspect the repository standard library
use repository paths; editing fixtures use owned temporary projects or document
overlays.

## Case selection

Select a suite target and filter by `module::case`, including nested module
prefixes where needed. The three directory entry points use their directory name:
`call_arguments/main.rs`, `import_aliases/main.rs` and
`record_construction/main.rs`.

```sh
cargo test --locked -p fpas-language-service
cargo test --locked -p fpas-language-service --test project_context
cargo test --locked -p fpas-language-service --test editor_queries -- call_arguments::signature_help_selects_reordered_named_var_parameters --exact
```

## Entry-point mapping

Each row maps a case file to its owning target. Its file stem supplies the module
prefix; directory entry points use their directory name. Each case file is
included once, including its nested modules.

| Case file | Target |
| --- | --- |
| `analysis.rs` | `analysis_state` |
| `case_coverage.rs` | `analysis_state` |
| `contracts.rs` | `analysis_state` |
| `document_store.rs` | `analysis_state` |
| `enum_values.rs` | `analysis_state` |
| `expression_type.rs` | `analysis_state` |
| `local_inference.rs` | `analysis_state` |
| `public_signatures.rs` | `analysis_state` |
| `query_cache.rs` | `analysis_state` |
| `routine_visibility.rs` | `analysis_state` |
| `unicode_character_literals.rs` | `analysis_state` |
| `project_index.rs` | `project_context` |
| `repository_context.rs` | `project_context` |
| `workspace_context.rs` | `project_context` |
| `acp.rs` | `editor_queries` |
| `api_lookup.rs` | `editor_queries` |
| `call_arguments/main.rs` | `editor_queries` |
| `discard.rs` | `editor_queries` |
| `intellisense.rs` | `editor_queries` |
| `intrinsic_std_api.rs` | `editor_queries` |
| `native_errors.rs` | `editor_queries` |
| `record_construction/main.rs` | `editor_queries` |
| `checked_navigation.rs` | `navigation_edits` |
| `import_aliases/main.rs` | `navigation_edits` |
| `navigation.rs` | `navigation_edits` |
| `navigation_resolution.rs` | `navigation_edits` |
| `refactoring.rs` | `navigation_edits` |
| `references_rename.rs` | `navigation_edits` |
| `rename_safety.rs` | `navigation_edits` |
| `semantic_tools.rs` | `navigation_edits` |
| `workspace_navigation.rs` | `navigation_edits` |
