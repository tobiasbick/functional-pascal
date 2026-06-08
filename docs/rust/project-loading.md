# Project loading (`fpas-project`)

How multi-file Functional Pascal programs are loaded before compile. Language rules: [`docs/pascal/10-projects.md`](../pascal/10-projects.md). Scope policy (no precompiled libs): [`docs/future/libraries.md`](../future/libraries.md).

## Crate layout

| Module | Responsibility |
|--------|----------------|
| `loading/own.rs` | Parse one `.fpasprj`, resolve `[sources]`, validate manifest |
| `test_sources.rs` | `*_test.fpas` naming; validate `kind = "test"` source lists |
| `dependencies.rs` | Merge deps; remap `link_meta` so transitive library origins and export policies stay correct |
| `loading/exports.rs` | Validate library `[exports].units` |
| `link/import_policy.rs` | Enforce cross-project unit imports for dependents |
| `workspace/` | `.fpasworkspace` members, name index, run discovery |
| `paths.rs` | Include/exclude globs and path resolution |
| `link/` | Parse units, build dependency graph, rewrite to linked `Program` |
| `link/library_check.rs` | Stub program linking all units for `fpas check` on libraries |

## Public API (`fpas_project::`)

- `load_project` — full project + dependency merge, unit-name validation
- `build_program` / `build_program_with_source_map` — link program entry + units
- `build_library_check_with_source_map` — link all library units for type-check only
- `load_workspace`, `discover_workspace_file`, `discover_run_project_in_workspace`, `discover_test_projects_in_workspace`
- `is_test_source_file` — basename ends with `_test.fpas` (shared with `fpas test` discovery)
- `resolve_workspace_dependency_paths` — map `dependencies.workspace` names to `.fpasprj` paths

## Tests

| Location | Coverage |
|----------|----------|
| `crates/fpas-project/tests/loading.rs` | Crate integration: deps, exclude, workspace resolve |
| `crates/fpas-cli/src/project/tests/` | Parser/link edge cases, manifest validation |
| `crates/fpas-cli/src/main_tests/projects/` | CLI `fpas` / `fpas check`, workspace run/discovery errors, transitive check |

When changing loading rules, extend `fpas-project` tests first, then CLI integration tests.
