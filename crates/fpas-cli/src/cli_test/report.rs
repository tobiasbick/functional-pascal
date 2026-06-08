//! Test result aggregation and summary output.
//!
//! **Documentation:** [`docs/future/test-framework/runner.md`](../../../docs/future/test-framework/runner.md)

use std::io::Write;

/// Outcome of running one test file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TestOutcome {
    Pass,
    AssertFailed,
    CompileError,
    RuntimeError,
}

impl TestOutcome {
    pub(super) fn is_failure(self) -> bool {
        !matches!(self, Self::Pass)
    }
}

/// Aggregated results for a test run.
#[derive(Debug, Default)]
pub(super) struct Summary {
    passed: usize,
    failed: usize,
    compile_errors: usize,
    runtime_errors: usize,
}

impl Summary {
    pub(super) fn record(&mut self, _path: &str, outcome: TestOutcome) {
        match outcome {
            TestOutcome::Pass => self.passed += 1,
            TestOutcome::AssertFailed => self.failed += 1,
            TestOutcome::CompileError => self.compile_errors += 1,
            TestOutcome::RuntimeError => self.runtime_errors += 1,
        }
    }

    /// Exit code: 2 compile, 3 runtime, 1 assert failure, 0 all pass.
    pub(super) fn exit_code(&self) -> i32 {
        if self.compile_errors > 0 {
            return 2;
        }
        if self.runtime_errors > 0 {
            return 3;
        }
        if self.failed > 0 {
            return 1;
        }
        0
    }
}

pub(super) fn print_summary(stderr: &mut dyn Write, summary: &Summary) -> std::io::Result<()> {
    let total = summary.passed + summary.failed + summary.compile_errors + summary.runtime_errors;
    let fail_count = summary.failed + summary.compile_errors + summary.runtime_errors;
    writeln!(
        stderr,
        "Summary: {} passed, {} failed ({} total)",
        summary.passed, fail_count, total
    )
}
