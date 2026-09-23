# Rust review implementation

This follow-up addresses the findings against `b3d3b84c` without changing FPAS language syntax or semantics. The original review remains a record of the state at that commit.

| Findings | Resolution |
| --- | --- |
| C1 | Escape only the literal project root in glob expansion. Project-loading tests cover bracketed and unmatched-bracket roots, decoy siblings, includes, and excludes. |
| C2, S1, D1 | Stage and validate unit sidecars with the existing atomic-write mechanism while retaining the unit lock. Failure-injection tests cover staging and commit errors and verify that the old sidecar remains readable. The unit documentation describes the replacement guarantee. |
| C3 | Correct the trailing-separator `BaseName` expectation and share the test across Windows and Unix. Root and empty inputs are covered. |
| C4 | Check count-derived minimum encoded sizes before reserving fixed-width tables. Variable-width tables grow as entries are decoded. Malformed count/payload fixtures exercise the rejection paths and measure actual allocation bytes; reverting the instruction precheck fails the 64 KiB budget. |
| C5 | Own benchmark descendants through an OS process group or job object and terminate them on timeout or parent exit. Timeout diagnostics retain termination, reaping, and pipe-drain errors together. Termination uses bounded polling with a direct-child fallback. Tests keep a real descendant and inherited pipes alive while both kill attempts fail, and verify combined termination, reap, and pipe diagnostics. |
| C6 | Search for the nearest source-owning manifest. A readable loose source stays available when an unrelated ancestor cannot be inspected; an explicit manifest still reports its error. Directory-inspection errors are distinguished from manifest and ownership errors. Regressions preserve invalid ancestor manifests, missing dependencies, and ambiguous owners. |
| P1 | Retain canonical exclusion keys in a set for one lookup per included file. Alias selection and wildcard exclusions under bracketed roots are covered. Operation-count tests measure 250/500 canonicalizations for 100/200 files with proportional exclusions. |
| P2 | Maintain occupied temporary registers incrementally and omit fixed locals above the search start. Allocation and compiler regressions cover register reuse and many locals. Generated-IR resource tests independently vary locals, result count, and live temporary count. |
| P3 | Remove the second project snapshot collection within one diagnostic request. Closed sources remain checked once per request so direct disk edits are visible without a file-watcher event. The broader cross-request cache is deliberately governed by that freshness rule. A forty-unit fixture measures one read per closed unit, including concurrent forks and disk/editor invalidation checks. |
| P4 | Clone direct interfaces only when compilation is required, and borrow read-only intrinsic string arguments. A warm 32-unit/496-import fixture records zero direct interface copies; successful parse allocation counts and bytes stay independent of input padding. |
| P5 | Compute canonical layout names once per layout before sorting and reuse them for assignment. A many-layout regression checks deterministic order. |
| S2 | Share record-field staging and emission after each construction form resolves its type. A regression checks initializer order. |
| S3 | Replace impossible runtime fallbacks with checked conversions or established invariants. |
| D2, D3 | Correct the intrinsic-copy and active format-limit comments. |

Same-drive before/after runs of the existing tooling and startup suites are recorded in [benchmark history](../../bench/history.md). The warm project build showed a repeatable improvement; cold builds varied widely, and the tooling suite did not establish a general speedup. Existing suite cases do not isolate exclusion lookup or register allocation, so no speedup is claimed for those paths. The follow-up [resource measurements](../../bench/rust-review-resources.md) provide deterministic counts and nine verified negative regression runs.

## Original implementation verification

- `cargo fmt --all -- --check`, `cargo build --workspace --locked --offline`, Clippy with all targets and features and `-D warnings`, and `git diff --check` passed.
- The complete Windows workspace test suite, including doc tests, passed with one test thread. A parallel run exposed a scheduler-sensitive existing one-second CLI timeout test; the test passed alone and in the complete serial run. A debugger step-event test failed once, then passed 100 direct repetitions and the complete serial run.
- On Unix, the three `BaseName` tests and the benchmark descendant-timeout regression passed.
- No `.fpas` source or FPAS language specification was changed.

## Follow-up verification

The follow-up corrects C5/C6 and supplies resource regressions for C4 and P1-P4.
It preserves language behavior and adds no `.fpas` sources. All builds ran
sequentially in the existing target directory with one Cargo build job.

- `cargo fmt --all -- --check` and `git diff --check` passed.
- `cargo build --workspace --locked --offline -j 1` passed.
- `cargo test --workspace --locked --offline -j 1 --no-fail-fast -- --test-threads=1 --nocapture`
  passed, including doc tests and the embedded FPAS suites.
- After the final discovery-boundary correction,
  `cargo test -p fpas-language-service -p fpas-lsp --locked --offline -j 1 --no-fail-fast -- --test-threads=1`
  and another workspace build passed. These tests verify that later diagnostics
  retain invalid-manifest, missing-dependency, and ambiguous-owner errors.
- `cargo clippy --workspace --all-targets --all-features --locked --offline -j 1 -- -D warnings`
  passed with the final discovery correction.
- Nine controlled negative runs failed their intended regression assertions;
  all temporarily reverted source files were restored before final verification.

These follow-up results are from Windows. The original Unix results above apply
to the earlier implementation only.
