//! Output regressions for the finite concurrency tutorials.

use super::{repo_root, support};

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
            "Task group close timed out\nCleanup joined; ownership released\n",
            ""
        )
    );
}
