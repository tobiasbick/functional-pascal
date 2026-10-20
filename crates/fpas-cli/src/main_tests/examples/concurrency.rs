//! Output regressions for the finite concurrency tutorials.

use super::{repo_root, support};

mod task_calls;

#[test]
fn example_task_group_workers_reports_result_errors_before_close() {
    let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(
        &[
            "run".into(),
            "examples/pascal/concurrency/task_group_workers.fpas".into(),
        ],
        &repo_root(),
    );
    let mut lines: Vec<_> = stdout.lines().collect();
    lines.sort_unstable();
    assert_eq!(exit, 0, "stderr: {stderr}");
    assert!(stderr.is_empty(), "{stderr}");
    assert_eq!(
        lines,
        ["A required setting is missing", "Background worker stopped"]
    );
}

#[test]
fn example_worker_pipeline_collects_every_square() {
    let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(
        &[
            "run".into(),
            "examples/pascal/concurrency/worker_pipeline.fpas".into(),
        ],
        &repo_root(),
    );
    assert_eq!((exit, stdout.as_str(), stderr.as_str()), (0, "650\n", ""));
}

#[test]
fn example_timed_close_retains_cleanup_until_release() {
    let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(
        &[
            "run".into(),
            "examples/pascal/concurrency/task_group_close_timeout.fpas".into(),
        ],
        &repo_root(),
    );
    assert_eq!(
        (exit, stdout.as_str(), stderr.as_str()),
        (
            0,
            "Std.Tasks.TaskError.Timeout: Operation timed out\n  Operation = \"Std.Tasks.CloseTaskGroupWithTimeout\"\nCleanup joined; ownership released\n",
            ""
        )
    );
}
