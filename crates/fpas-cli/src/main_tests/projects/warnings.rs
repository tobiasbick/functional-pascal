use super::*;

#[test]
fn run_cli_emits_warning_for_program_source_file_and_still_runs() {
    let cwd = create_temp_dir("run-program-source-warning");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses App.Util, Std.Console;\nbegin\n  WriteLn(GetValue())\nend.\n",
    );
    write_text(
        &cwd.join("src/util.fpas"),
        "unit App.Util;\nfunction GetValue(): integer;\nbegin\n  return 42\nend;\n",
    );
    write_text(&cwd.join("src/tool.fpas"), "program Tool;\nbegin\nend.\n");

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "42\n");
    assert!(stderr_output.contains("warning:"));
    assert!(stderr_output.contains("declares `program Tool` and was skipped"));
}

#[test]
fn run_cli_emits_warning_for_duplicate_source_file_and_still_runs() {
    let cwd = create_temp_dir("run-duplicate-source-warning");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(
        &project_file,
        "src/main.fpas",
        &["src/util.fpas", "src/*.fpas", "src/util.fpas"],
    );
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses App.Util, Std.Console;\nbegin\n  WriteLn(GetValue())\nend.\n",
    );
    write_text(
        &cwd.join("src/util.fpas"),
        "unit App.Util;\nfunction GetValue(): integer;\nbegin\n  return 7\nend;\n",
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "7\n");
    assert!(stderr_output.contains("warning: Duplicate source file"));
}
