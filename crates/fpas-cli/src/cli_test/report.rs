//! Test result aggregation and summary output.
//!
//! **Documentation:** [`docs/pascal/std/testing/test.md`](../../../docs/pascal/std/testing/test.md)

use std::io::Write;

use serde::{Deserialize, Serialize};

/// Outcome of running one test file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(super) enum TestOutcome {
    Pass,
    Skipped,
    NotRun,
    AssertFailed,
    CompileError,
    RuntimeError,
    TimedOut,
}

impl TestOutcome {
    pub(super) fn is_failure(self) -> bool {
        !matches!(self, Self::Pass | Self::Skipped | Self::NotRun)
    }

    fn status_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Skipped => "skipped",
            Self::NotRun => "not_run",
            Self::AssertFailed => "assert_failed",
            Self::CompileError => "compile_error",
            Self::RuntimeError => "runtime_error",
            Self::TimedOut => "timed_out",
        }
    }
}

/// One executed test file and its outcome.
#[derive(Debug, Clone)]
pub(super) struct TestCaseResult {
    pub file: String,
    pub outcome: TestOutcome,
}

/// Aggregated results for a test run.
#[derive(Debug, Default)]
pub(super) struct Summary {
    passed: usize,
    skipped: usize,
    not_run: usize,
    failed: usize,
    compile_errors: usize,
    runtime_errors: usize,
    timed_out: usize,
    cases: Vec<TestCaseResult>,
}

impl Summary {
    pub(super) fn record(&mut self, path: &str, outcome: TestOutcome) {
        match outcome {
            TestOutcome::Pass => self.passed += 1,
            TestOutcome::Skipped => self.skipped += 1,
            TestOutcome::NotRun => self.not_run += 1,
            TestOutcome::AssertFailed => self.failed += 1,
            TestOutcome::CompileError => self.compile_errors += 1,
            TestOutcome::RuntimeError => self.runtime_errors += 1,
            TestOutcome::TimedOut => self.timed_out += 1,
        }
        self.cases.push(TestCaseResult {
            file: path.to_string(),
            outcome,
        });
    }

    /// Exit code: 2 compile, 3 runtime, 1 assert failure or strict skip, 0 all pass.
    pub(super) fn exit_code(&self, strict: bool) -> i32 {
        if self.compile_errors > 0 {
            return 2;
        }
        if self.runtime_errors > 0 {
            return 3;
        }
        if self.timed_out > 0 {
            return 3;
        }
        if self.failed > 0 {
            return 1;
        }
        if strict && self.skipped > 0 {
            return 1;
        }
        0
    }
}

#[derive(Serialize)]
struct JsonReport<'a> {
    version: u32,
    summary: JsonSummary,
    tests: Vec<JsonTestCase<'a>>,
}

#[derive(Serialize)]
struct JsonSummary {
    passed: usize,
    skipped: usize,
    not_run: usize,
    failed: usize,
    compile_errors: usize,
    runtime_errors: usize,
    timed_out: usize,
    total: usize,
}

#[derive(Serialize)]
struct JsonTestCase<'a> {
    file: &'a str,
    status: &'static str,
}

pub(super) fn print_summary(stderr: &mut dyn Write, summary: &Summary) -> std::io::Result<()> {
    let total = summary.passed
        + summary.skipped
        + summary.not_run
        + summary.failed
        + summary.compile_errors
        + summary.runtime_errors
        + summary.timed_out;
    let fail_count =
        summary.failed + summary.compile_errors + summary.runtime_errors + summary.timed_out;
    if summary.skipped > 0 || summary.not_run > 0 {
        writeln!(
            stderr,
            "Summary: {} passed, {} skipped, {} not run, {} failed ({} total)",
            summary.passed, summary.skipped, summary.not_run, fail_count, total
        )
    } else {
        writeln!(
            stderr,
            "Summary: {} passed, {} failed ({} total)",
            summary.passed, fail_count, total
        )
    }
}

/// Writes a machine-readable JSON report to stdout for CI consumers.
pub(super) fn print_json_report(stdout: &mut dyn Write, summary: &Summary) -> std::io::Result<()> {
    let total = summary.passed
        + summary.skipped
        + summary.not_run
        + summary.failed
        + summary.compile_errors
        + summary.runtime_errors
        + summary.timed_out;
    let report = JsonReport {
        version: 1,
        summary: JsonSummary {
            passed: summary.passed,
            skipped: summary.skipped,
            not_run: summary.not_run,
            failed: summary.failed,
            compile_errors: summary.compile_errors,
            runtime_errors: summary.runtime_errors,
            timed_out: summary.timed_out,
            total,
        },
        tests: summary
            .cases
            .iter()
            .map(|case| JsonTestCase {
                file: case.file.as_str(),
                status: case.outcome.status_str(),
            })
            .collect(),
    };
    let json = serde_json::to_string_pretty(&report).map_err(|error| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("failed to serialize JSON test report: {error}"),
        )
    })?;
    writeln!(stdout, "{json}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_json_report_serializes_summary_and_cases() {
        let mut summary = Summary::default();
        summary.record("alpha_test.fpas", TestOutcome::Pass);
        summary.record("beta_test.fpas", TestOutcome::AssertFailed);
        summary.record("gamma_test.fpas", TestOutcome::Skipped);

        let mut stdout = Vec::new();
        print_json_report(&mut stdout, &summary).expect("write json");
        let text = String::from_utf8(stdout).expect("utf-8");
        assert!(text.contains("\"version\": 1"));
        assert!(text.contains("\"passed\": 1"));
        assert!(text.contains("\"skipped\": 1"));
        assert!(text.contains("\"failed\": 1"));
        assert!(text.contains("\"status\": \"pass\""));
        assert!(text.contains("\"status\": \"assert_failed\""));
        assert!(text.contains("\"status\": \"skipped\""));
        assert!(text.contains("\"file\": \"alpha_test.fpas\""));
    }

    #[test]
    fn exit_code_treats_skipped_as_success_unless_strict() {
        let mut summary = Summary::default();
        summary.record("skip_test.fpas", TestOutcome::Skipped);
        assert_eq!(summary.exit_code(false), 0);
        assert_eq!(summary.exit_code(true), 1);
    }
}
