# `fpas-cli` review follow-up

Classification: CLI and project tooling. Preserve current FPAS discovery and automation contracts unless the matching docs change.
Status: all findings open.

| ID | Priority | Evidence | Finding and impact | Implementation direction | Required regression |
| --- | --- | --- | --- | --- | --- |
| CLI-01 | P1 | `crates/fpas-cli/src/cli_test/timeout.rs:45` | After a timeout and grace period, the CLI always calls blocking `join()`. A VM thread stuck in a non-cooperative operation makes `fpas test --timeout` hang forever. | Isolate timed tests in a terminable process, or otherwise provide a lifecycle that never blocks beyond the timeout contract. Do not detach a thread that can retain process resources without an explicit design. | A non-cooperative test must produce bounded failure and cleanup. |
| CLI-02 | P2 | `crates/fpas-cli/src/cli_check.rs:54,68`, `src/project_build.rs:84` | `fpas check <dir>` builds every `.fpas` file as an independent program. Pure Units and programs importing sibling Units fail despite a valid combined source set. | Classify the directory once, construct a shared unit graph, and check program roots against it. | Directory with a program plus sibling Unit, a pure Unit set, and multiple related sources. |
| CLI-03 | P2 | `crates/fpas-cli/src/cli_paths.rs:49`, `src/cli_test/discover.rs:74` | Recursive discovery silently drops `read_dir` and entry errors, so check/fmt/test can skip subtrees and still succeed. | Replace duplicate walkers with one fallible traversal that reports the exact path and aborts. | Unreadable directory and failing entry cases for check, fmt, and test. |
| CLI-04 | P2 | `crates/fpas-cli/src/cli_input/mod.rs:93,154` | Value-taking flags consume the following option token as their value, producing misleading later errors. | Centralize value consumption and reject EOF or a known option token as a missing value. | Every value-taking option followed by another flag, including `--std-lib --help` and `--script --fail-fast`. |
| CLI-05 | P2 | `crates/fpas-cli/src/cli_test/runner.rs:16` | JSON report and summary write errors are discarded; a missing/truncated contracted report can still exit zero. | Propagate output failures and return nonzero with best-effort stderr diagnostics. | Inject a writer failure for JSON, summary, list, and help output as applicable. |

## Implementation notes

CLI-03 and CLI-04 are concrete simplification opportunities: one fallible path walker and one option-value helper remove duplicated error-prone policy. Update focused command help and automation contracts when observable exit/output behavior changes.

Targeted verification should include crate tests plus relevant `fpas test tests/` or runner-suite coverage. Then run the shared workspace verification from the index.
