# Crate Code Review

Review date: 2026-07-04.

Scope: all current Cargo workspace crates under `crates/*`. This is a read-only code review captured as Markdown; no Rust or FPAS behavior was changed.

## Method

- Inspected workspace metadata, crate structure, large files, public API shape, diagnostics, error handling, and test organization.
- Ran Clippy per crate where possible with `cargo clippy -p <crate> --all-targets -- -D warnings`.
- Ran the workspace command `cargo clippy --workspace --all-targets -- -D warnings`; it now passes after resolving the `fpas-vm` lint blockers.
- Treated generated or intentionally test-only patterns as lower priority unless they block tooling or violate the repository structure rules.

## Highest Priority Findings

1. `fpas-std`, `fpas-cli`, `fpas-fmt`: several implementation or test files exceed the repository's 400/500 LOC structure threshold. See the per-crate files.
2. `fpas-vm`: resolved on 2026-07-04 — workspace Clippy was blocked by `TuiState`'s manual `Default` implementation and dead test helper functions; VM structure findings are also resolved. See [fpas-vm.md](fpas-vm.md).

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
