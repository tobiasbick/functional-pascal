use super::*;

#[test]
fn run_cli_executes_program_project_main_file() {
    let cwd = create_temp_dir("run-program-project");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/**/*.fpas"]);
    write_text(&cwd.join("src/main.fpas"), "program Main;\nbegin\nend.\n");

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0);
    assert!(stdout_output.is_empty());
    assert!(stderr_output.is_empty());
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
        "unit App.Util;\nuses App.Math;\nfunction Double(X: integer): integer;\nbegin\n  return Add(X, X)\nend;\n",
    );
    write_text(
        &cwd.join("src/math.fpas"),
        "unit App.Math;\nfunction Add(A: integer; B: integer): integer;\nbegin\n  return A + B\nend;\n",
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
        "unit App.Parts;\n\nconst\n  Message: string := 'Hello from unit';\n",
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "Hello from unit\n");
    assert!(stderr_output.is_empty());
}
