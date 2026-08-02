# `fpas-bench` review follow-up

Classification: CLI/tooling and benchmark infrastructure. No language change expected.
Status: all findings done.

| ID | Priority | Evidence | Finding and impact | Implementation direction | Required regression |
| --- | --- | --- | --- | --- | --- |
| BENCH-01 | P1 | `crates/fpas-bench/src/results.rs:251-283`, `src/main.rs:49` | A current benchmark missing from the baseline receives no delta and cannot trigger `--fail-on-regression`. A snapshot from another group can therefore produce a false green result. | Require exactly one baseline row for every current benchmark; report missing and duplicate IDs. Persist and validate the selected group if snapshots remain group-specific. | Compare disjoint and partially overlapping groups; missing or duplicate baseline IDs must fail with actionable diagnostics. |
| BENCH-02 | P1 | `crates/fpas-bench/src/main.rs:111-121` | `NaN` passes the non-negative threshold check and makes every regression comparison false. Infinity also disables useful gating. | Require `threshold_pct.is_finite() && threshold_pct >= 0.0`. | Cover `NaN`, positive/negative infinity, negative finite, zero, and a normal threshold through parser and CLI exit behavior. |
| BENCH-03 | P2 | `crates/fpas-bench/src/results.rs:260-265` | A zero-millisecond baseline maps every later duration to `0%`, hiding arbitrary slowdown. | Define `0 -> 0` explicitly and treat `0 -> positive` as a regression or invalid baseline. | Boundary cases for zero/nonzero baseline and current duration. |
| BENCH-04 | P2 | `crates/fpas-bench/src/suite.rs:71-102` | A custom `CARGO_TARGET_DIR` is inherited by Cargo, but the harness still searches `<repo>/target/release/fpas`. | Resolve Cargo's effective target directory, preferably from metadata, and use it consistently. | Run with a temporary custom target directory and verify executable discovery. |
| BENCH-05 | P2 | `crates/fpas-bench/src/results.rs:139-154` | `record_history` truncates the existing committed history before a complete replacement is guaranteed. | Write a same-directory temporary file, flush as required, then publish with the repository's agreed atomic-replacement strategy. Apply the same policy to snapshots where useful. | Inject write/publish failure and prove the previous history remains intact. |
| BENCH-06 | P3 | `crates/fpas-bench/src/suite.rs:14-15,63-66`, `src/main.rs:154-155` | Help and unknown-group diagnostics omit the existing `concurrency` group. | Derive group names from the loaded suite or keep one authoritative definition. | Help and invalid-group tests include all configured groups. |
| BENCH-07 | P3 | `crates/fpas-bench/src/main.rs:18-24,123` | `--help` is modeled as an error and likely exits with status 1. Confidence is lower because no explicit contract was found. | Represent help as a successful parser outcome; reserve errors for invalid invocation. | Binary-level help stdout/stderr and exit-code test. |
| BENCH-08 | P3 | `crates/fpas-bench/src/suite.rs:112-124` | A hung benchmark blocks the harness indefinitely. | Decide and document whether each spec needs a timeout; if yes, terminate the process and retain captured diagnostics. | Stub benchmark that never exits; assert bounded failure and cleanup. |

## Completion notes

The fixes change benchmark-harness correctness and reliability, not the performance of FPAS workloads. No speedup was claimed and no benchmark history entry was recorded.

## BENCH-02 completion record

Completed on 2026-08-02.

- Implementation: `parse_args` now rejects non-finite and negative threshold percentages before repository discovery or benchmark execution.
- Regressions: parser tests cover `NaN`, positive and negative infinity, negative finite values, zero, and a normal value. Binary-level tests verify exit code 1 and the actionable diagnostic for every invalid class.
- Docs: `docs/bench/README.md` now states that thresholds must be finite and non-negative. Normative `docs/pascal/` pages are unchanged because FPAS behavior did not change.
- Verification: `cargo fmt`; `cargo test -p fpas-bench`; `cargo clippy -p fpas-bench --all-targets --locked -- -D warnings`; `cargo build`; `cargo test --workspace`.

## BENCH-03 through BENCH-08 completion record

Completed on 2026-08-02.

- BENCH-03: comparison defines `0 -> 0` as no change and rejects `0 -> positive` with an actionable invalid-measurement error. Boundary tests cover both paths and the existing nonzero calculation.
- BENCH-04: executable discovery obtains Cargo's effective target directory from `cargo metadata`; a test supplies a temporary `CARGO_TARGET_DIR` to the metadata process and verifies the resolved directory.
- BENCH-05: snapshots and committed history now share `results/publication.rs`, backed by `atomic-write-file`. Content is staged in the destination directory, flushed, and atomically committed. Injected write and commit failures prove that the previous artifact remains intact and temporary files are cleaned up.
- BENCH-06: group names are derived from the loaded suite in first-appearance order. Unit and binary tests require `vm`, `concurrency`, and `tui` in help and unknown-group diagnostics.
- BENCH-07: argument parsing represents help as a successful outcome. Binary coverage requires exit code 0, help on stdout, empty stderr, configured groups, and examples.
- BENCH-08: every suite entry has a required nonzero `timeout_ms`. The runner drains both output pipes concurrently, terminates and reaps an expired process, and includes captured output in the error. A real nonterminating child fixture proves bounded failure and cleanup.
- Structure: argument parsing moved to `arguments.rs`; comparison and publication live under `results/`; executable discovery and bounded process execution live under `suite/`. All Rust files remain below 300 lines.
- Docs: `docs/bench/README.md`, `docs/bench/suite.toml`, and the benchmark agent skill describe current groups, target discovery, atomic persistence, zero-baseline behavior, and timeouts. Normative `docs/pascal/` pages are unchanged because FPAS behavior did not change.
- Verification: `cargo fmt`; `cargo test -p fpas-bench` (29 unit tests and 7 binary tests); `cargo clippy -p fpas-bench --all-targets --locked -- -D warnings`; `cargo build`; `cargo test --workspace`.

## BENCH-01 completion record

Completed on 2026-08-02.

- Implementation: snapshots now persist the selected group. `ComparisonBaseline` validates group identity and duplicate ids before release-binary setup, then requires one baseline result for every current benchmark before producing complete comparison rows.
- Structure: comparison validation, delta calculation, rows, and regression gating moved from `results.rs` into `results/comparison.rs`; persistence and history remain in `results.rs`.
- Regressions: unit tests cover disjoint and partially overlapping baselines, duplicate ids, mismatched groups, and a valid comparison. A binary-level test proves a group mismatch exits with an actionable diagnostic before release setup.
- Docs: `docs/bench/README.md` documents group-bound snapshots, complete baseline coverage, and the need to recreate older local snapshots. Normative `docs/pascal/` pages are unchanged because FPAS behavior did not change.
- Verification: `cargo fmt`; `cargo test -p fpas-bench`; `cargo clippy -p fpas-bench --all-targets --locked -- -D warnings`; `cargo build`; `cargo test --workspace`.
