# `fpas-bench` review follow-up

Classification: CLI/tooling and benchmark infrastructure. No language change expected.
Status: all findings open.

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

## Implementation notes

Start with BENCH-01 and BENCH-02 because they undermine the regression gate itself. BENCH-05 should reuse the artifact-publication decision made for build, bundle, std, and unit code. Measure any claimed speedup; none was established by this review.

Targeted verification should cover unit tests for comparison logic plus binary-level CLI tests with a temporary repository and stub `fpas` executable.
