//! Root-import errors identify the main source and the imported name's exclusive range.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md`

use super::*;
use serde_json::{Value, json};
use std::path::PathBuf;

const MISSING_SOURCE: &str =
    "program Demo; // 😀\r\nuses Std.Console, Demo.Missing as Missing;\r\nbegin\r\nend.\r\n";
const PRIVATE_SOURCE: &str = "program Demo;\nuses Library.Hidden as Hidden;\nbegin\nend.\n";

fn assert_import_record(stderr: &str, main: &Path, code: &str, start: u32, end: u32) {
    let records: Vec<Value> = stderr
        .lines()
        .map(|line| serde_json::from_str(line).expect("diagnostic JSON"))
        .collect();
    assert_eq!(records.len(), 1, "{stderr}");
    let record = &records[0];
    assert_eq!(record["code"], code, "{stderr}");
    assert_eq!(record["severity"], "error");
    assert_eq!(record["phase"], "project");
    let actual = Path::new(record["source"].as_str().expect("main source path"));
    assert_eq!(
        fs::canonicalize(actual).expect("diagnostic path"),
        fs::canonicalize(main).expect("main path")
    );
    assert_eq!(
        record["location"],
        json!({"source_id": 0, "start": {"line": 2, "column": start}, "end": {"line": 2, "column": end}})
    );
    assert!(record["hint"].is_string());
}

fn assert_commands(
    cwd: &Path,
    input: &Path,
    main: &Path,
    commands: &[&str],
    code: &str,
    range: (u32, u32),
    standard_library: Option<&Path>,
) {
    for command in commands {
        let mut args = vec![
            (*command).into(),
            "--diagnostics".into(),
            "json".into(),
            input.to_string_lossy().into_owned(),
        ];
        if let Some(root) = standard_library {
            args.extend(["--std-lib".into(), root.to_string_lossy().into_owned()]);
        }
        let (status, stdout, stderr) = support::run_cli_args_and_capture_output(&args, cwd);
        assert_eq!(status, 1, "{command}: {stderr}");
        assert_eq!(stdout, "", "{command}: {stdout}");
        assert_import_record(&stderr, main, code, range.0, range.1);
    }
}

fn project(cwd: &Path, source: &str, private_library: bool) -> (PathBuf, PathBuf) {
    let main = cwd.join("app/src/main.fpas");
    write_text(&main, source);
    let manifest = cwd.join("app/demo.fpasprj");
    let dependencies = if private_library {
        "[dependencies]\nprojects = ['../library/library.fpasprj']\n"
    } else {
        ""
    };
    write_text(
        &manifest,
        &format!(
            "[project]\nname = 'demo'\nkind = 'program'\nmain = 'src/main.fpas'\n[sources]\ninclude = ['src/*.fpas']\n{dependencies}"
        ),
    );
    if private_library {
        write_text(
            &cwd.join("library/library.fpasprj"),
            "[project]\nname = 'library'\nkind = 'library'\n[sources]\ninclude = ['*.fpas']\n[exports]\nunits = ['Library.Api']\n",
        );
        write_text(
            &cwd.join("library/api.fpas"),
            "unit Library.Api; end unit;\n",
        );
        write_text(
            &cwd.join("library/hidden.fpas"),
            "unit Library.Hidden; end unit;\n",
        );
    }
    (manifest, main)
}

#[test]
fn missing_root_import_in_source_check_and_run_retains_its_name_range() {
    let cwd = create_temp_dir("root-import-source");
    let main = cwd.join("main.fpas");
    write_text(&main, MISSING_SOURCE);
    assert_commands(
        &cwd,
        &main,
        &main,
        &["check", "run"],
        "FP4115",
        (19, 31),
        None,
    );
    fs::remove_dir_all(&cwd).expect("fixture cleanup");
}

#[test]
fn missing_root_import_in_project_and_workspace_check_build_and_run_is_located() {
    let cwd = create_temp_dir("root-import-project");
    let (manifest, main) = project(&cwd, MISSING_SOURCE, false);
    let workspace = cwd.join("demo.fpasworkspace");
    write_text(
        &workspace,
        "[workspace]\nname = 'demo'\nmembers = ['app/demo.fpasprj']\n",
    );
    for input in [&manifest, &workspace] {
        assert_commands(
            &cwd,
            input,
            &main,
            &["check", "build", "run"],
            "FP4115",
            (19, 31),
            None,
        );
    }
    fs::remove_dir_all(&cwd).expect("fixture cleanup");
}

#[test]
fn private_root_import_in_project_and_workspace_check_build_and_run_is_located() {
    let cwd = create_temp_dir("root-import-private");
    let (manifest, main) = project(&cwd, PRIVATE_SOURCE, true);
    let workspace = cwd.join("demo.fpasworkspace");
    write_text(
        &workspace,
        "[workspace]\nname = 'demo'\nmembers = ['app/demo.fpasprj']\n",
    );
    for input in [&manifest, &workspace] {
        assert_commands(
            &cwd,
            input,
            &main,
            &["check", "build", "run"],
            "FP4116",
            (6, 20),
            None,
        );
    }
    fs::remove_dir_all(&cwd).expect("fixture cleanup");
}

#[test]
fn private_standard_library_root_import_in_source_check_and_run_is_located() {
    let cwd = create_temp_dir("root-import-private-std");
    let main = cwd.join("main.fpas");
    let root = cwd.join("lib");
    write_text(
        &main,
        "program Demo;\nuses Std.Hidden as Hidden;\nbegin\nend.\n",
    );
    write_text(
        &root.join("stdlib.fpasprj"),
        "[project]\nname = 'stdlib'\nkind = 'library'\n[sources]\ninclude = ['Std/*.fpas']\n[exports]\nunits = ['Std.Api']\n",
    );
    write_text(&root.join("Std/Api.fpas"), "unit Std.Api; end unit;\n");
    write_text(
        &root.join("Std/Hidden.fpas"),
        "unit Std.Hidden; end unit;\n",
    );
    assert_commands(
        &cwd,
        &main,
        &main,
        &["check", "run"],
        "FP4116",
        (6, 16),
        Some(&root),
    );
    fs::remove_dir_all(&cwd).expect("fixture cleanup");
}

#[test]
fn root_import_text_and_json_agree_on_source_position_code_and_help() {
    let cwd = create_temp_dir("root-import-parity");
    let main = cwd.join("main.fpas");
    write_text(&main, MISSING_SOURCE);
    let path = main.to_string_lossy().into_owned();
    let (status, _, text) =
        support::run_cli_args_and_capture_output(&["check".into(), path.clone()], &cwd);
    assert_eq!(status, 1);
    assert!(
        text.starts_with(&format!("{path}:2:19: error[FP4115]: Unknown unit")),
        "{text}"
    );
    assert!(text.contains("help:"));
    let (status, _, json) = support::run_cli_args_and_capture_output(
        &["check".into(), "--diagnostics".into(), "json".into(), path],
        &cwd,
    );
    assert_eq!(status, 1);
    assert_import_record(&json, &main, "FP4115", 19, 31);
    fs::remove_dir_all(&cwd).expect("fixture cleanup");
}

#[test]
fn root_import_failures_in_a_shared_test_graph_identify_each_entry_source() {
    let cwd = create_temp_dir("root-import-test-entries");
    let manifest = cwd.join("tests.fpasprj");
    write_text(
        &manifest,
        "[project]\nname = 'tests'\nkind = 'test'\n[sources]\ninclude = ['*_test.fpas']\n",
    );
    let first = cwd.join("first_test.fpas");
    let second = cwd.join("second_test.fpas");
    for path in [&first, &second] {
        write_text(path, MISSING_SOURCE);
    }
    for jobs in ["1", "2"] {
        let (status, stdout, stderr) = support::run_cli_args_and_capture_output(
            &[
                "test".into(),
                "--diagnostics".into(),
                "json".into(),
                "--report".into(),
                "json".into(),
                "--jobs".into(),
                jobs.into(),
                manifest.to_string_lossy().into_owned(),
            ],
            &cwd,
        );
        assert_eq!(status, 2, "{stderr}");
        let report: Value = serde_json::from_str(&stdout).expect("test report");
        assert_eq!(report["summary"]["compile_errors"], 2);
        let records: Vec<&str> = stderr.lines().collect();
        assert_eq!(records.len(), 2, "{stderr}");
        for path in [&first, &second] {
            let record = records
                .iter()
                .find(|line| {
                    line.contains(
                        path.file_name()
                            .expect("filename")
                            .to_str()
                            .expect("UTF-8 filename"),
                    )
                })
                .expect("entry diagnostic");
            assert_import_record(record, path, "FP4115", 19, 31);
        }
    }
    fs::remove_dir_all(&cwd).expect("fixture cleanup");
}
