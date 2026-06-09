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
        return prepared
            .into_iter()
            .map(|test| run_prepared_test(test, script_override, timeout))
            .collect();
    }

    let script_path = script_override.map(Path::to_path_buf);
    let stop = Arc::new(AtomicBool::new(false));
    let mut results = Vec::with_capacity(prepared.len());
    let mut batch_start = 0_usize;

    while batch_start < prepared.len() {
        if fail_fast && stop.load(Ordering::Relaxed) {
            break;
        }

        let batch_end = (batch_start + worker_count).min(prepared.len());
        let mut handles = Vec::with_capacity(batch_end - batch_start);
        for test in prepared[batch_start..batch_end].iter().cloned() {
            let script_path = script_path.clone();
            let stop = Arc::clone(&stop);
            handles.push(thread::spawn(move || {
                if fail_fast && stop.load(Ordering::Relaxed) {
                    return None;
                }
                let result = run_prepared_test(test, script_path.as_deref(), timeout);
                if result.outcome.is_failure() {
                    stop.store(true, Ordering::Relaxed);
                }
                Some(result)
            }));
        }

        for handle in handles {
            if let Ok(Some(result)) = handle.join() {
                results.push(result);
            }
        }

        batch_start = batch_end;
    }

    results.sort_by_key(|result| result.index);
    results
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
}
