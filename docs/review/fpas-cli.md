# `fpas-cli` review follow-up

Classification: CLI and project tooling. Preserve current FPAS discovery and automation contracts unless the matching docs change.
Status: all findings completed 2026-08-03.

| ID | Priority | Evidence | Finding and impact | Implementation direction | Required regression |
| --- | --- | --- | --- | --- | --- |
| CLI-01 | P1 | Done | Timed VM runs now use a private `fpas` worker with a Ready/Start handshake, bounded file protocol, process-tree termination, reaping, and scratch cleanup. | Blocking `Std.Time.Sleep`, a long-lived `Std.Proc` descendant, timeout classification, and cleanup are covered by integration tests. |
| CLI-02 | P2 | Done | Directory checks classify one shared source set, check Units in memory without publishing `.fpascu`, and validate every Program against the sibling Unit graph. | Pure Units, Program plus Unit, multiple Programs, and absence of new sidecars are covered. |
| CLI-03 | P2 | Done | Check, Fmt, and Test share one fallible deterministic walker. It skips `target` directories and symbolic links and reports `read_dir`, entry, and file-type failures with their path. | Injected traversal failures and sorted/non-following behavior are covered. |
| CLI-04 | P2 | Done | Option parsing moved to `cli_input/options.rs`; one value helper rejects EOF and known option tokens for all seven value-taking flags without banning other hyphen-leading values. | Table-driven option-boundary tests and a hyphen-leading filter regression are covered. |
| CLI-05 | P2 | Done | Contracted stdout and summary writes now return nonzero on failure with best-effort diagnostics through `cli_output.rs`. | Immediate and partial failures cover Help, Version, Fmt output/list, test list, JSON, and summary output. |

## Implementation notes

The implementation preserves FPAS syntax and semantics. Observable CLI behavior is documented in
`docs/pascal/program-structure/cli.md` and timeout lifecycle behavior in
`docs/pascal/std/testing/test.md`.

Verification:

- `cargo fmt --all -- --check`
- `cargo build --workspace`
- `cargo clippy -p fpas-cli -p fpas-build --all-targets -- -D warnings`
- `cargo test -p fpas-build --lib`
- `cargo test -p fpas-cli -- --test-threads=1` — 393 unit, 2 native-executable, and 2 timeout integration tests passed
- `cargo test --workspace -- --test-threads=1`

The CLI crate is run serially because its integration fixtures intentionally replace the shared
`target/debug/lib` standard-library copy; parallel unit-test threads can otherwise race on that
fixture rather than exercising product behavior.
