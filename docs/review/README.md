# Crate Code Review

Review date: 2026-07-04.

Scope: all current Cargo workspace crates under `crates/*`. This review is captured as Markdown and tracks both findings and remediation progress.

## Method

- Inspected workspace metadata, crate structure, large files, public API shape, diagnostics, error handling, and test organization.
- Ran Clippy per crate where possible with `cargo clippy -p <crate> --all-targets -- -D warnings`.
- Ran the workspace command `cargo clippy --workspace --all-targets -- -D warnings`; it passes after resolving the `fpas-vm` lint blockers.
- Treated generated or intentionally test-only patterns as lower priority unless they block tooling or violate the repository structure rules.

## Remediation Progress (2026-07-04)

- `fpas-vm`: structure splits and Clippy blockers resolved. See [fpas-vm.md](fpas-vm.md).
- `fpas-std`: runtime module splits, `std_units/symbols` registry split, and `tui/tests` split resolved. No `fpas-std` source file now exceeds 400 lines. See [fpas-std.md](fpas-std.md).
- `fpas-cli`, `fpas-compiler`: package Clippy is unblocked now that `fpas-vm` passes strict linting.

## Highest Priority Findings

1. `fpas-cli`: argument parsing and test runner files exceed the structure threshold. Package Clippy passes; structure cleanup is next. See [fpas-cli.md](fpas-cli.md).
2. `fpas-fmt`: emitter modules are large but below the hard split threshold; defer splits until the next formatter change. See [fpas-fmt.md](fpas-fmt.md).
3. `fpas-compiler`: oversized compiler test files remain low priority. See [fpas-compiler.md](fpas-compiler.md).

## Per-Crate Notes

- [fpas-bytecode](fpas-bytecode.md)
- [fpas-diagnostics](fpas-diagnostics.md)
- [fpas-lexer](fpas-lexer.md)
- [fpas-parser](fpas-parser.md)
- [fpas-sema](fpas-sema.md)
- [fpas-compiler](fpas-compiler.md)
- [fpas-std](fpas-std.md)
- [fpas-vm](fpas-vm.md)
- [fpas-fmt](fpas-fmt.md)
- [fpas-project](fpas-project.md)
- [fpas-cli](fpas-cli.md)
