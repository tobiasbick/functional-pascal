use super::*;

#[test]
fn run_cli_executes_program_project_main_file() {
    let cwd = create_temp_dir("run-program-project");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/**/*.fpas"]);
    write_text(&cwd.join("src/main.fpas"), "program Main;\nbegin\nend.\n");

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    let artifact_exists = cwd.join("app.fpascp").is_file();
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0);
    assert!(artifact_exists, "project run must publish app.fpascp");
    assert!(stdout_output.is_empty());
    assert!(stderr_output.is_empty());
}

#[test]
fn run_cli_rebuilds_stale_program_artifact_before_execution() {
    let cwd = create_temp_dir("run-rebuild-program-artifact");
    let project_file = cwd.join("app.fpasprj");
    let main_file = cwd.join("src/main.fpas");
    let artifact_file = cwd.join("app.fpascp");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/**/*.fpas"]);
    write_text(
        &main_file,
        "program Main;\nuses Std.Console;\nbegin\n  WriteLn(1)\nend.\n",
    );

    let first = support::run_cli_and_capture_output(&project_file, &cwd);
    let first_artifact = fs::read(&artifact_file).expect("first run must publish artifact");
    write_text(
        &main_file,
        "program Main;\nuses Std.Console;\nbegin\n  WriteLn(2)\nend.\n",
    );
    let second = support::run_cli_and_capture_output(&project_file, &cwd);
    let second_artifact = fs::read(&artifact_file).expect("second run must retain artifact");
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(first.0, 0, "stderr: {}", first.2);
    assert_eq!(first.1, "1\n");
    assert_eq!(second.0, 0, "stderr: {}", second.2);
    assert_eq!(second.1, "2\n");
    assert_ne!(first_artifact, second_artifact);
}

#[test]
fn run_cli_executes_compiled_program_without_project_sources() {
    let cwd = create_temp_dir("run-compiled-program");
    let project_file = cwd.join("app.fpasprj");
    let artifact_file = cwd.join("app.fpascp");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/**/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses Std.Console;\nbegin\n  WriteLn('from image')\nend.\n",
    );

    let build = support::run_cli_args_and_capture_output(
        &[
            String::from("build"),
            project_file.to_string_lossy().into_owned(),
        ],
        &cwd,
    );
    fs::remove_file(&project_file).expect("manifest must be removed");
    fs::remove_dir_all(cwd.join("src")).expect("sources must be removed");
    let run = support::run_cli_args_and_capture_output(
        &[
            String::from("run"),
            artifact_file.to_string_lossy().into_owned(),
        ],
        &cwd,
    );
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(build.0, 0, "stderr: {}", build.2);
    assert_eq!(run.0, 0, "stderr: {}", run.2);
    assert_eq!(run.1, "from image\n");
    assert!(run.2.is_empty());
}

#[test]
fn run_cli_rejects_corrupt_compiled_program() {
    let cwd = create_temp_dir("run-corrupt-compiled-program");
    let artifact_file = cwd.join("broken.fpascp");
    write_text(&artifact_file, "not a compiled program");

    let (exit_code, _, stderr_output) = support::run_cli_args_and_capture_output(
        &[
            String::from("run"),
            artifact_file.to_string_lossy().into_owned(),
        ],
        &cwd,
    );
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(stderr_output.contains("Cannot run compiled program"));
    assert!(stderr_output.contains("invalid `.fpascp` magic header"));
    assert!(stderr_output.contains("fpas build"));
}

#[test]
fn run_cli_rejects_old_compiled_program_with_rebuild_help() {
    let cwd = create_temp_dir("run-old-compiled-program");
    let artifact_file = cwd.join("old.fpascp");
    let mut bytes = b"FPASCP\0\0".to_vec();
    bytes.extend_from_slice(
        &fpas_program::PROGRAM_FORMAT_VERSION
            .saturating_sub(1)
            .to_le_bytes(),
    );
    fs::write(&artifact_file, bytes).expect("old-format fixture must be written");

    let (exit_code, _, stderr_output) = support::run_cli_args_and_capture_output(
        &[
            String::from("run"),
            artifact_file.to_string_lossy().into_owned(),
        ],
        &cwd,
    );
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(stderr_output.contains("unsupported `.fpascp` format version"));
    assert!(stderr_output.contains("this runtime requires version"));
    assert!(stderr_output.contains("Rebuild the `.fpascp`"));
    assert!(stderr_output.contains("fpas build"));
}

#[test]
fn run_cli_executes_multi_file_project_end_to_end() {
    let cwd = create_temp_dir("run-multifile-project");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses App.Util, Std.Console;\nbegin\n  WriteLn(Double(3))\nend.\n",
    );
    write_text(
        &cwd.join("src/util.fpas"),
        "unit App.Util;\nuses App.Math;\npublic function Double(X: integer): integer;\nbegin\n  return Add(X, X)\nend;\n",
    );
    write_text(
        &cwd.join("src/math.fpas"),
        "unit App.Math;\npublic function Add(A: integer; B: integer): integer;\nbegin\n  return A + B\nend;\n",
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "6\n");
    assert!(stderr_output.is_empty());
}

#[test]
fn run_cli_shares_constants_via_unit_instead_of_include() {
    let cwd = create_temp_dir("run-project-shared-unit");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses App.Parts, Std.Console;\nbegin\n  WriteLn(Message)\nend.\n",
    );
    write_text(
        &cwd.join("src/parts.fpas"),
        "unit App.Parts;\n\npublic const\n  Message: string := 'Hello from unit';\n",
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "Hello from unit\n");
    assert!(stderr_output.is_empty());
}

#[test]
fn run_cli_rejects_directory_path() {
    let cwd = create_temp_dir("run-source-directory");
    write_text(&cwd.join("main.fpas"), "program Main;\nbegin\nend.\n");

    let (exit_code, _, stderr_output) = support::run_cli_args_and_capture_output(
        &[String::from("run"), cwd.to_string_lossy().to_string()],
        &cwd,
    );
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(stderr_output.contains("Cannot run directory"));
}
