//! Checks implemented generic forms in `docs/pascal/tools/fmt-style.md`.

use super::{repo_root, support};
use crate::test_support::{create_temp_dir, write_text};
use std::fs;
use std::path::PathBuf;

const HANDBOOK: &str = include_str!("../../../../../docs/pascal/tools/fmt-style.md");

fn type_declarations() -> String {
    let section = HANDBOOK
        .split("## More examples — other types (snippet)")
        .nth(1)
        .expect("other-types section")
        .split("## Types (summary)")
        .next()
        .expect("other-types section body");
    let blocks = section.split("```").skip(1).step_by(2).collect::<Vec<_>>();
    assert_eq!(blocks.len(), 1, "one other-types example");
    let (language, source) = blocks[0].split_once('\n').expect("fenced language");
    assert_eq!(language.trim(), "pascal");
    source.replace("\r\n", "\n")
}

fn generic_program() -> String {
    let summary = HANDBOOK
        .lines()
        .find(|line| line.starts_with("- Generic routines:"))
        .expect("generic routine summary");
    let signature = summary.split('`').nth(1).expect("routine signature");
    // Supply the routine body and calls for the documented signature and inferred types.
    format!(
        "program Doc;\nuses Std.Console as Console;\n{}\n\
         {signature}\nbegin\n  return Value;\nend function;\n\
         begin\n  Console.WriteLn(Identity(42));\n  Console.WriteLn(Identity('hi'));\nend.\n",
        type_declarations()
    )
}

fn write_example(source: &str) -> (PathBuf, PathBuf) {
    let dir = create_temp_dir("formatter-generics");
    let path = dir.join("generics.fpas");
    write_text(&path, source);
    (dir, path)
}

#[test]
fn documented_generic_types_and_routines_pass_cli_check() {
    let (dir, path) = write_example(&generic_program());
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
fn documented_generic_routine_infers_integer_and_string_arguments() {
    let (dir, path) = write_example(&generic_program());
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
        (0, "42\nhi\n", "")
    );
}

#[test]
fn documented_types_match_formatter_output() {
    let source = format!("program Doc;\n\n{}\nbegin\nend.\n", type_declarations());
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
