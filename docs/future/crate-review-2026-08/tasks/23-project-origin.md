# Task 23 — Do not retag library sources as `Own`

Status: open
Severity: P1
Difficulty: medium
Language gate: no
Depends on: none

## Goal

If the same `.fpas` file is both a library unit and listed in the consumer’s `[sources]`, origin stays **Library**. `[exports]` still applies. The library can still `uses` that unit.

## Spec

[`docs/pascal/program-structure/projects.md`](../../../pascal/program-structure/projects.md): the same file may appear via a library and via `[sources]`. Exports gate `Own → Library`.

## Bug

`crates/fpas-project/src/dependencies.rs` `mark_own_source_origins` overwrites any merged file that `same_file`-matches the consumer include list. Then `unit_graph/resolve.rs`: `Own → Own` allows a program to `uses` a non-exported library unit; `Library → Own` is denied so the library cannot import its own file.

There is a symlink/alias origin test in `project_integrity.rs`, but nothing that loads consumer `[sources]` + dependency sharing one physical file.

## Fix

Do not overwrite an existing `Library` origin with `Own`. Only mark `Own` for files that were not already merged from a dependency. If both apply, **Library wins** (exports stay meaningful). Confirm with the spec text; if the spec wants Own-wins, stop and report — the review’s bug is export bypass, so Library-wins is the fix that matches `[exports]`.

## Tests

Project test: two manifests, shared path, library does **not** export the unit → consumer `uses` that unit must fail. Library-internal `uses` still ok.

Also the reverse false-reject: library unit importing the shared file must not get `Library → Own` denial.

## Verify

```text
cargo test -p fpas-project
cargo fmt
```

## Done when

- Shared files keep Library origin.
- Export bypass is gone.
- Docs unchanged unless projects.md needs a one-line origin rule.
