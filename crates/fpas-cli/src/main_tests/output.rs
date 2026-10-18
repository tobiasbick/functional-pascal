use super::*;
use crate::test_support::FailingWriter;

#[test]
fn help_and_version_fail_when_stdout_cannot_be_written() {
    let cwd = create_temp_dir("contract-output-run");

    for (args, expected) in [
        (vec![String::from("--help")], "Cannot write help to stdout"),
        (
            vec![String::from("--version")],
            "Cannot write version to stdout",
        ),
    ] {
        for writer in [FailingWriter::immediately(), FailingWriter::after(3)] {
            let mut stderr = Vec::new();
            let exit_code = run_cli(&args, &cwd, Box::new(writer), &mut stderr, false);
            let stderr = String::from_utf8(stderr).expect("stderr must be UTF-8");

            assert_eq!(exit_code, 1);
            assert!(stderr.contains(expected), "unexpected stderr: {stderr}");
        }
    }

    fs::remove_dir_all(&cwd).expect("temp directory must be removed");
}

#[test]
fn fmt_stdout_fails_when_formatted_source_is_only_partially_written() {
    let cwd = create_temp_dir("contract-output-fmt-stdout");
    let source = cwd.join("main.fpas");
    write_text(&source, "program Main; begin end.");
    let args = [
        String::from("fmt"),
        String::from("--stdout"),
        source.to_string_lossy().into_owned(),
    ];
    for writer in [FailingWriter::immediately(), FailingWriter::after(8)] {
        let mut stderr = Vec::new();
        let exit_code = run_cli(&args, &cwd, Box::new(writer), &mut stderr, false);
        let stderr = String::from_utf8(stderr).expect("stderr must be UTF-8");

        assert_eq!(exit_code, 1);
        assert!(
            stderr.contains("Cannot write formatted source to stdout"),
            "unexpected stderr: {stderr}"
        );
    }
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");
}

#[test]
fn fmt_list_fails_when_changed_file_list_cannot_be_written() {
    let cwd = create_temp_dir("contract-output-fmt-list");
    let source = cwd.join("main.fpas");
    write_text(&source, "program Main; begin end.");
    let args = [
        String::from("fmt"),
        String::from("--check"),
        String::from("--list"),
        source.to_string_lossy().into_owned(),
    ];
    for writer in [FailingWriter::immediately(), FailingWriter::after(8)] {
        let mut stderr = Vec::new();
        let exit_code = run_cli(&args, &cwd, Box::new(writer), &mut stderr, false);
        let stderr = String::from_utf8(stderr).expect("stderr must be UTF-8");

        assert_eq!(exit_code, 1);
        assert!(
            stderr.contains("Cannot write changed file list to stdout"),
            "unexpected stderr: {stderr}"
        );
    }
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");
}

#[test]
fn query_fails_when_its_json_document_is_only_partially_written() {
    let cwd = create_temp_dir("contract-output-query");
    write_text(&cwd.join("main.fpas"), "program T; begin discard 1; end.");
    let args = [
        "query".into(),
        "type".into(),
        "main.fpas:1:26".into(),
        "--std-lib".into(),
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../lib")
            .to_string_lossy()
            .into_owned(),
    ];
    for writer in [FailingWriter::immediately(), FailingWriter::after(8)] {
        let mut stderr = Vec::new();
        let code = run_cli(&args, &cwd, Box::new(writer), &mut stderr, false);
        assert_eq!(code, 1);
        let record: serde_json::Value =
            serde_json::from_slice(&stderr).expect("structured write failure");
        assert!(
            record["message"]
                .as_str()
                .expect("message")
                .contains("Cannot write query output")
        );
    }
}
