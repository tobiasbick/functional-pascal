//! Long-import golden checks extracted from `docs/pascal/tools/fmt-style.md`.

use super::{EXIT_WOULD_CHANGE, create_temp_dir, run_cli_args_and_capture_output, write_text};
use std::fs;
use std::path::PathBuf;

fn long_import_example() -> String {
    let markdown = include_str!("../../../../../docs/pascal/tools/fmt-style.md");
    let blocks = markdown
        .split("```")
        .skip(1)
        .step_by(2)
        .filter(|block| block.lines().any(|line| line == "program LongUses;"))
        .collect::<Vec<_>>();
    assert_eq!(blocks.len(), 1, "one complete long-import golden");
    let (language, source) = blocks[0].split_once('\n').expect("fenced language");
    assert_eq!(language.trim(), "pascal");
    source.replace("\r\n", "\n")
}

fn write_example(source: &str) -> (PathBuf, PathBuf) {
    let dir = create_temp_dir("fmt-handbook-imports");
    let path = dir.join("long-uses.fpas");
    write_text(&path, source);
    (dir, path)
}

#[test]
fn documented_long_imports_match_existing_golden_and_cli_output() {
    let source = long_import_example();
    let expected = include_str!("../../../../fpas-fmt/tests/golden/long_uses.expected.fpas")
        .replace("\r\n", "\n");
    assert_eq!(source, expected, "handbook and formatter golden must agree");
    let (dir, path) = write_example(&source);
    let (exit, stdout, stderr) = run_cli_args_and_capture_output(
        &[
            "fmt".into(),
            "--stdout".into(),
            path.to_string_lossy().into_owned(),
        ],
        &dir,
    );
    fs::remove_dir_all(dir).expect("remove fixture");
    assert_eq!(
        (exit, stdout.as_str(), stderr.as_str()),
        (0, source.as_str(), "")
    );
}

#[test]
fn documented_long_imports_pass_cli_format_check_without_changes() {
    let source = long_import_example();
    let (dir, path) = write_example(&source);
    let (exit, stdout, stderr) = run_cli_args_and_capture_output(
        &[
            "fmt".into(),
            "--check".into(),
            path.to_string_lossy().into_owned(),
        ],
        &dir,
    );
    let unchanged = fs::read_to_string(&path).expect("read checked fixture");
    fs::remove_dir_all(dir).expect("remove fixture");
    assert_eq!((exit, stdout.as_str(), stderr.as_str()), (0, "", ""));
    assert_eq!(unchanged, source, "format check must not change its input");
}

#[test]
fn original_single_line_imports_fail_check_and_format_to_documented_output() {
    let expected = long_import_example();
    assert!(expected.contains("uses\n  "), "wrapped import clause");
    let source = expected.replacen("uses\n  ", "uses ", 1);
    let (dir, path) = write_example(&source);
    let (check_exit, _, check_stderr) = run_cli_args_and_capture_output(
        &[
            "fmt".into(),
            "--check".into(),
            path.to_string_lossy().into_owned(),
        ],
        &dir,
    );
    let unchanged = fs::read_to_string(&path).expect("read checked fixture");
    let (fmt_exit, stdout, fmt_stderr) = run_cli_args_and_capture_output(
        &[
            "fmt".into(),
            "--stdout".into(),
            path.to_string_lossy().into_owned(),
        ],
        &dir,
    );
    fs::remove_dir_all(dir).expect("remove fixture");
    assert_eq!(check_exit, EXIT_WOULD_CHANGE, "{check_stderr}");
    assert_eq!(unchanged, source, "failed check must not change its input");
    assert_eq!(
        (fmt_exit, stdout.as_str(), fmt_stderr.as_str()),
        (0, expected.as_str(), "")
    );
}
