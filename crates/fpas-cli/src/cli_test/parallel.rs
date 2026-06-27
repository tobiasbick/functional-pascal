//! Parallel test execution for `fpas test`.
//!
//! **Documentation:** [`docs/future/test-framework/runner.md`](../../../docs/future/test-framework/runner.md)

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use super::report::TestOutcome;
use super::run::{LinkContext, run_single_test_capture};

/// One test ready to execute on a worker thread.
#[derive(Clone)]
pub(super) struct PreparedTest {
    pub index: usize,
    pub path: PathBuf,
    pub display: String,
    pub link: Option<LinkContext>,
}

/// Buffered output from one test run, ordered by discovery index.
pub(super) struct IndexedTestResult {
    pub index: usize,
    pub display: String,
    pub outcome: TestOutcome,
    pub output: String,
}

/// Resolves the worker count from `--jobs` and the number of tests.
pub(super) fn effective_job_count(requested: usize, test_count: usize) -> usize {
    let requested = if requested == 0 {
        thread::available_parallelism()
            .map(|count| count.get())
            .unwrap_or(1)
    } else {
        requested
    };
    requested.max(1).min(test_count.max(1))
}

/// Runs prepared tests with up to `jobs` concurrent workers.
pub(super) fn run_tests_parallel(
    prepared: Vec<PreparedTest>,
    jobs: usize,
    script_override: Option<&Path>,
    timeout: Option<Duration>,
    fail_fast: bool,
) -> Vec<IndexedTestResult> {
    if prepared.is_empty() {
        return Vec::new();
    }

    let worker_count = effective_job_count(jobs, prepared.len());
    if worker_count <= 1 {
        return run_tests_sequential(prepared, script_override, timeout, fail_fast);
    }

    let script_path = script_override.map(Path::to_path_buf);
    let stop = Arc::new(AtomicBool::new(false));
    let mut results = Vec::with_capacity(prepared.len());
    let mut batch_start = 0_usize;

    while batch_start < prepared.len() {
        if fail_fast && stop.load(Ordering::Relaxed) {
            results.extend(prepared[batch_start..].iter().cloned().map(not_run_result));
            break;
        }

        let batch_end = (batch_start + worker_count).min(prepared.len());
        let mut handles = Vec::with_capacity(batch_end - batch_start);
        for test in prepared[batch_start..batch_end].iter().cloned() {
            let script_path = script_path.clone();
            let stop = Arc::clone(&stop);
            handles.push(thread::spawn(move || {
                if fail_fast && stop.load(Ordering::Relaxed) {
                    return not_run_result(test);
                }
                let result = run_prepared_test(test, script_path.as_deref(), timeout);
                if fail_fast && result.outcome.is_failure() {
                    stop.store(true, Ordering::Relaxed);
                }
                result
            }));
        }

        for handle in handles {
            if let Ok(result) = handle.join() {
                results.push(result);
            }
        }

        batch_start = batch_end;
    }

    results.sort_by_key(|result| result.index);
    results
}

fn run_tests_sequential(
    prepared: Vec<PreparedTest>,
    script_override: Option<&Path>,
    timeout: Option<Duration>,
    fail_fast: bool,
) -> Vec<IndexedTestResult> {
    let mut results = Vec::<IndexedTestResult>::with_capacity(prepared.len());
    let mut stop = false;
    for test in prepared {
        if fail_fast && stop {
            results.push(not_run_result(test));
            continue;
        }
        let result = run_prepared_test(test, script_override, timeout);
        if fail_fast && result.outcome.is_failure() {
            stop = true;
        }
        results.push(result);
    }
    results
}

fn not_run_result(test: PreparedTest) -> IndexedTestResult {
    IndexedTestResult {
        index: test.index,
        display: test.display.clone(),
        outcome: TestOutcome::NotRun,
        output: format!("  ---  {} (not run, --fail-fast)\n", test.display),
    }
}

fn run_prepared_test(
    test: PreparedTest,
    script_override: Option<&Path>,
    timeout: Option<Duration>,
) -> IndexedTestResult {
    let (outcome, output_bytes) =
        run_single_test_capture(&test.path, test.link.as_ref(), script_override, timeout);
    IndexedTestResult {
        index: test.index,
        display: test.display,
        outcome,
        output: String::from_utf8_lossy(&output_bytes).into_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{create_temp_dir, write_text};
    use std::thread;

    #[test]
    fn effective_job_count_caps_at_test_count() {
        assert_eq!(effective_job_count(8, 3), 3);
    }

    #[test]
    fn effective_job_count_never_returns_zero() {
        assert_eq!(effective_job_count(1, 0), 1);
    }

    #[test]
    fn two_threads_run_assert_programs() {
        let dir = create_temp_dir("fpas-parallel-vm");
        let first = dir.join("a_test.fpas");
        let second = dir.join("b_test.fpas");
        write_text(
            &first,
            "program A;\nuses Std.Test;\nbegin AssertTrue(true) end.",
        );
        write_text(
            &second,
            "program B;\nuses Std.Test;\nbegin AssertTrue(true) end.",
        );

        let handles = [first, second]
            .map(|path| thread::spawn(move || run_single_test_capture(&path, None, None, None)));

        for handle in handles {
            let (outcome, _) = handle.join().expect("worker join");
            assert_eq!(outcome, TestOutcome::Pass);
        }
    }

    #[test]
    fn run_tests_parallel_completes_small_batch() {
        let dir = create_temp_dir("fpas-parallel-batch");
        let first = dir.join("one_test.fpas");
        let second = dir.join("two_test.fpas");
        write_text(
            &first,
            "program O;\nuses Std.Test;\nbegin AssertTrue(true) end.",
        );
        write_text(
            &second,
            "program T;\nuses Std.Test;\nbegin AssertEquals(2, 1 + 1) end.",
        );

        let prepared = vec![
            PreparedTest {
                index: 0,
                path: first,
                display: "one_test.fpas".to_string(),
                link: None,
            },
            PreparedTest {
                index: 1,
                path: second,
                display: "two_test.fpas".to_string(),
                link: None,
            },
        ];

        let results = run_tests_parallel(prepared, 2, None, None, false);
        assert_eq!(results.len(), 2);
        assert!(
            results
                .iter()
                .all(|result| result.outcome == TestOutcome::Pass)
        );
    }

    #[test]
    fn run_tests_parallel_fail_fast_records_not_run_tests() {
        let dir = create_temp_dir("fpas-parallel-fail-fast");
        let pass = dir.join("pass_test.fpas");
        let fail = dir.join("fail_test.fpas");
        let later = dir.join("later_test.fpas");
        write_text(
            &pass,
            "program P;\nuses Std.Test;\nbegin AssertTrue(true) end.",
        );
        write_text(
            &fail,
            "program F;\nuses Std.Test;\nbegin AssertTrue(false) end.",
        );
        write_text(
            &later,
            "program L;\nuses Std.Test;\nbegin AssertTrue(true) end.",
        );

        let prepared = vec![
            PreparedTest {
                index: 0,
                path: pass,
                display: "pass_test.fpas".to_string(),
                link: None,
            },
            PreparedTest {
                index: 1,
                path: fail,
                display: "fail_test.fpas".to_string(),
                link: None,
            },
            PreparedTest {
                index: 2,
                path: later,
                display: "later_test.fpas".to_string(),
                link: None,
            },
        ];

        let results = run_tests_parallel(prepared, 1, None, None, true);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].outcome, TestOutcome::Pass);
        assert_eq!(results[1].outcome, TestOutcome::AssertFailed);
        assert_eq!(results[2].outcome, TestOutcome::NotRun);
        assert!(results[2].output.contains("not run, --fail-fast"));
    }
}
