# Rust workspace review

Implementation follow-up: [resolution and verification](implementation.md).

Reviewed on 2026-09-23 against commit `b3d3b84c3c1952364a545d786548eb617e3b596a`. The worktree was clean when review started. This report records findings and proposed follow-up work. No Rust implementation, language specification, or permanent tests were changed.

## Scope and evidence

The request names no individual crate, so the review covers the 22 workspace crates. Repository-wide inventory, file sizes, Rust documentation paths, formatting, build, lint and test checks are combined with source walkthroughs. Detailed inspection concentrates on project loading, incremental builds, artifact decoding/publication, register allocation, linking, standard-library argument handling, language-service caching, and benchmark process cleanup. This is not a line-by-line audit of every compiler, debugger, parser, or VM instruction.

Every finding distinguishes a reproduced failure, a consequence visible in source, or a performance candidate. Performance costs below are algorithmic observations, not measured speedups. Proposed changes preserve existing language behavior. Any proposal that requires changing language semantics still needs an explicit decision.

Priorities: P1 means incorrect source selection or similarly urgent correctness work; P2 means a functional, resource, or scaling problem; P3 means localized maintenance or documentation work.

## Findings

| ID | Priority | Finding | Evidence |
| --- | --- | --- | --- |
| C1 | P1 | Glob expansion treats the project root as pattern syntax and can select sibling sources | Reproduced through `fpas_project::expand_path_glob` |
| C2 | P2 | Windows unit publication has a missing-destination interval and discards rollback errors | Source-confirmed; failure injection still needed |
| C3 | P2 | A Unix-only path test expects behavior contrary to the implementation | Source review; Unix execution still needed |
| C4 | P2 | Program decoding allocates from item counts before proving that bytes exist | Source-confirmed resource amplification |
| C5 | P2 | Benchmark process-tree timeout test fails in this Windows environment | Observed test failure; cause not isolated |
| C6 | P2 | Loose-file discovery depends on readable ancestor directories | Test and probe reproduce an access-denied failure |
| P1 | P2 | Source exclusions repeatedly canonicalize paths in a nested loop | Source-confirmed filesystem work |
| P2 | P2 | Register allocation rebuilds an occupied-register tree for every temporary | Source-confirmed scaling candidate |
| P3 | P2 | Semantic cache hits still read all closed project sources from disk | Source-confirmed I/O candidate |
| P4 | P3 | Build reuse and read-only intrinsics perform avoidable deep copies | Source-confirmed copies |
| P5 | P3 | Layout sort keys repeatedly scan definitions and allocate names | Source-confirmed scaling candidate |
| S1 | P3 | Artifact publication mechanisms have diverged | Source comparison; overlaps C2 |
| S2 | P3 | Record construction repeats staging and emission logic | Source comparison |
| S3 | P3 | Some defensive fallbacks hide established invariants | Source review |
| D1 | P2 | Atomic sidecar documentation overstates Windows publication guarantees | Source/documentation mismatch; overlaps C2 |
| D2 | P3 | Intrinsic argument documentation promises fewer copies than implementation performs | Source/documentation mismatch |
| D3 | P3 | Active format limits are described as reserved for a future codec | Source/documentation mismatch |

These rows overlap where one implementation issue also affects documentation or structure. They are not 17 independent bugs.

## Reports

- [Correctness and test gaps](correctness-and-tests.md)
- [Performance candidates](performance.md)
- [Structure, duplication, and defensive code](structure-and-defensive-code.md)
- [Documentation](documentation.md)

## Verification

Temporary probes and command logs live under the ignored `.temp-data/rust-review/` directory and are not part of this report's tracked deliverables.

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed |
| `cargo build --workspace --locked --offline` | Passed after the tests finished. An earlier concurrent attempt could not replace the running `fpas.exe` on Windows. |
| `cargo test --workspace --locked --offline` | Stopped at C5 in `fpas-bench` |
| `cargo test --workspace --locked --offline --no-fail-fast` | Completed all targets: 3,146 passed, 2 failed, 0 ignored, summed from Rust test-result lines, including doc tests. Failures are C5 and C6. |
| Targeted repeat of `source_without_metadata_uses_loose_context` | Failed again; a separate public-API probe reported an access-denied discovery issue |
| `cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings` | Failed at `crates/fpas-vm/src/vm/hosted/random/state.rs:85`, lint `chunks_exact_to_as_chunks`. This is a lint-gate failure, not evidence of incorrect random output. |
| Explicit Rust `docs/pascal/*.md` target existence | No missing files found |
| `git diff --check` and report reference checks | Passed |

The CLI's existing FPAS suite integration tests, including collection, concurrency and TUI suites, passed during the complete run. No separate full `fpas test tests/` invocation was made. No permanent regression tests were added in this documentation-only review.

The glob probe created ordinary directories named `plain`, `demo[1]`, and `demo1`, each containing a distinct `.fpas` file. Its relevant output was:

```text
plain: ["plain.fpas"]
demo[1]: []
demo1: ["demo1.fpas"]
bracket root with sibling present: ["demo1.fpas"]
```

No benchmark measurements, Unix test execution, allocation profiling, or crash injection were performed. Missing regression tests are described in the finding that motivates them. Existing coverage is acknowledged rather than inferred from filenames alone.

## Suggested order

1. Fix and regress C1. It can change which source tree is compiled.
2. Resolve C2 and D1 together, with publication failure injection.
3. Investigate C5 and correct C3 so the baseline test suite is trustworthy on both platforms.
4. Bound C4 allocations by available encoded bytes.
5. Measure P1, P2, and P3 on representative large projects before selecting the next optimization.
6. Apply localized duplication and documentation cleanup while touching the relevant modules.
