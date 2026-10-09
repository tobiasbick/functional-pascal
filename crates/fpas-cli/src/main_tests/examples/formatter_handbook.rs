//! Semantic and formatter checks for the control-flow example in `docs/pascal/tools/fmt-style.md`.

use super::{repo_root, support};
use crate::test_support::{create_temp_dir, write_text};
use std::fs;
use std::path::PathBuf;

fn control_flow_example() -> String {
    let markdown = include_str!("../../../../../docs/pascal/tools/fmt-style.md");
    let blocks = markdown
        .split("```")
        .skip(1)
        .step_by(2)
        .filter(|block| block.lines().any(|line| line == "program ControlFlowDemo;"))
        .collect::<Vec<_>>();
    assert_eq!(blocks.len(), 1, "one complete control-flow example");
    let (_, source) = blocks[0].split_once('\n').expect("fenced language");
    source.replace("\r\n", "\n")
}

fn write_example(source: &str) -> (PathBuf, PathBuf) {
    let dir = create_temp_dir("formatter-handbook");
    let path = dir.join("control-flow.fpas");
    write_text(&path, source);
    (dir, path)
}

#[test]
fn documented_control_flow_passes_cli_check() {
    let (dir, path) = write_example(&control_flow_example());
    let (exit, _, stderr) = support::run_cli_args_and_capture_output(
        &[
            "check".into(),
            "--std-lib".into(),
            repo_root().join("lib").to_string_lossy().into_owned(),
            path.to_string_lossy().into_owned(),
        ],
        &repo_root(),
    );
    fs::remove_dir_all(dir).expect("remove fixture");
    assert_eq!(exit, 0, "{stderr}");
    assert!(stderr.is_empty(), "{stderr}");
}

#[test]
fn documented_control_flow_runs_with_expected_output() {
    let (dir, path) = write_example(&control_flow_example());
    let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(
        &[
            "run".into(),
            "--std-lib".into(),
            repo_root().join("lib").to_string_lossy().into_owned(),
            path.to_string_lossy().into_owned(),
        ],
        &repo_root(),
    );
    fs::remove_dir_all(dir).expect("remove fixture");
    assert_eq!(
        (exit, stdout.as_str(), stderr.as_str()),
        (0, "positive\nother\n1\n2\n3\n0\n1\n2\n", "")
    );
}

#[test]
fn documented_control_flow_matches_formatter_output() {
    let source = control_flow_example();
    let (dir, path) = write_example(&source);
    let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(
        &[
            "fmt".into(),
            "--stdout".into(),
            path.to_string_lossy().into_owned(),
        ],
        &repo_root(),
    );
    fs::remove_dir_all(dir).expect("remove fixture");
    assert_eq!(
        (exit, stdout.as_str(), stderr.as_str()),
        (0, source.as_str(), "")
    );
}

#[test]
fn original_const_control_flow_reports_fp3005() {
    let source = control_flow_example();
    assert!(
        source.contains("var X: integer := 5;"),
        "mutable loop binding"
    );
    let source = source.replacen("var X: integer := 5;", "const X: integer := 5;", 1);
    let (dir, path) = write_example(&source);
    let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(
        &[
            "check".into(),
            "--std-lib".into(),
            repo_root().join("lib").to_string_lossy().into_owned(),
            path.to_string_lossy().into_owned(),
        ],
        &repo_root(),
    );
    fs::remove_dir_all(dir).expect("remove fixture");
    assert_eq!(exit, 1, "{stderr}");
    assert!(stdout.is_empty(), "{stdout}");
    assert!(stderr.contains("error[FP3005]"), "{stderr}");
    assert!(stderr.contains("Cannot assign to `X`"), "{stderr}");
    assert_eq!(stderr.matches("error[").count(), 1, "{stderr}");
}
