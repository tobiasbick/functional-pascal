//! Console imports in complete introductory programs from `docs/pascal/`.

use super::{repo_root, support};
use crate::test_support::{create_temp_dir, write_text};
use std::fs;
use std::path::PathBuf;

struct Example {
    name: &'static str,
    source: String,
    output: &'static str,
}

fn fenced_program(markdown: &str) -> String {
    let blocks = markdown.split("```").skip(1).step_by(2).collect::<Vec<_>>();
    assert_eq!(blocks.len(), 1, "one introductory program fence");
    let (language, source) = blocks[0].split_once('\n').expect("fenced language");
    assert_eq!(language.trim(), "pascal");
    source.replace("\r\n", "\n")
}

fn introductory_examples() -> [Example; 2] {
    let keywords = include_str!("../../../../../docs/pascal/getting-started/keywords.md");
    let keyword_example = keywords
        .split("## Example")
        .nth(1)
        .expect("keyword example section")
        .split("## See also")
        .next()
        .expect("keyword example body");
    let formatter = include_str!("../../../../../docs/pascal/tools/fmt-style.md");
    let minimal_example = formatter
        .split("### Program — minimal")
        .nth(1)
        .expect("minimal program section")
        .split("### Program — with `uses`")
        .next()
        .expect("minimal program body");
    [
        Example {
            name: "keyword-demo",
            source: fenced_program(keyword_example),
            output: "same keywords, different casing\n",
        },
        Example {
            name: "minimal-console",
            source: fenced_program(minimal_example),
            output: "Hello, World!\n",
        },
    ]
}

fn write_example(name: &str, source: &str) -> (PathBuf, PathBuf) {
    let dir = create_temp_dir(name);
    let path = dir.join("introductory.fpas");
    write_text(&path, source);
    (dir, path)
}

fn check_or_run(example: &Example, source: &str, command: &str) -> (i32, String, String) {
    let (dir, path) = write_example(example.name, source);
    let result = support::run_cli_args_and_capture_output(
        &[
            command.into(),
            "--std-lib".into(),
            repo_root().join("lib").to_string_lossy().into_owned(),
            path.to_string_lossy().into_owned(),
        ],
        &repo_root(),
    );
    fs::remove_dir_all(dir).expect("remove fixture");
    result
}

#[test]
fn documented_introductory_programs_pass_cli_check() {
    for example in introductory_examples() {
        let (exit, _, stderr) = check_or_run(&example, &example.source, "check");
        assert_eq!(exit, 0, "{}\n{stderr}", example.name);
        assert!(stderr.is_empty(), "{}\n{stderr}", example.name);
    }
}

#[test]
fn documented_introductory_programs_run_with_expected_output() {
    for example in introductory_examples() {
        let (exit, stdout, stderr) = check_or_run(&example, &example.source, "run");
        assert_eq!(
            (exit, stdout.as_str(), stderr.as_str()),
            (0, example.output, ""),
            "{}",
            example.name
        );
    }
}

#[test]
fn removing_introductory_console_import_reports_fp3003() {
    for example in introductory_examples() {
        assert!(
            example
                .source
                .lines()
                .any(|line| line.trim().eq_ignore_ascii_case("uses Std.Console;")),
            "{}: explicit Console import",
            example.name
        );
        let source = example
            .source
            .lines()
            .filter(|line| !line.trim().eq_ignore_ascii_case("uses Std.Console;"))
            .collect::<Vec<_>>()
            .join("\n");
        let (exit, stdout, stderr) = check_or_run(&example, &source, "check");
        assert_eq!(exit, 1, "{}\n{stderr}", example.name);
        assert!(stdout.is_empty(), "{}\n{stdout}", example.name);
        assert!(
            stderr.contains("error[FP3003]"),
            "{}\n{stderr}",
            example.name
        );
        assert!(
            stderr.contains("Unknown procedure"),
            "{}\n{stderr}",
            example.name
        );
        assert_eq!(
            stderr.matches("error[").count(),
            1,
            "{}\n{stderr}",
            example.name
        );
    }
}

#[test]
fn documented_minimal_program_matches_formatter_output() {
    let [_, example] = introductory_examples();
    let (dir, path) = write_example(example.name, &example.source);
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
        (0, example.source.as_str(), "")
    );
}
