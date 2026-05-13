use super::*;

#[test]
fn diamond_dependency_graph() {
    let cwd = create_temp_dir("run-diamond-deps");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses App.A, App.B, Std.Console;\nbegin\n  WriteLn(FromA() + FromB())\nend.\n",
    );
    write_text(
        &cwd.join("src/a.fpas"),
        "unit App.A;\nuses App.Shared;\nfunction FromA(): integer;\nbegin\n  return Base() + 1\nend;\n",
    );
    write_text(
        &cwd.join("src/b.fpas"),
        "unit App.B;\nuses App.Shared;\nfunction FromB(): integer;\nbegin\n  return Base() + 10\nend;\n",
    );
    write_text(
        &cwd.join("src/shared.fpas"),
        "unit App.Shared;\nfunction Base(): integer;\nbegin\n  return 100\nend;\n",
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "211\n");
}

#[test]
fn three_unit_cyclic_dependency() {
    let cwd = create_temp_dir("run-three-cycle");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses App.A;\nbegin\nend.\n",
    );
    write_text(&cwd.join("src/a.fpas"), "unit App.A;\nuses App.B;\n");
    write_text(&cwd.join("src/b.fpas"), "unit App.B;\nuses App.C;\n");
    write_text(&cwd.join("src/c.fpas"), "unit App.C;\nuses App.A;\n");

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("Cyclic unit dependency detected"),
        "expected cycle error, got: {stderr_output}"
    );
}

#[test]
fn self_import_reports_cycle() {
    let cwd = create_temp_dir("run-self-import");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses App.A;\nbegin\nend.\n",
    );
    write_text(&cwd.join("src/a.fpas"), "unit App.A;\nuses App.A;\n");

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("Cyclic unit dependency detected"),
        "expected cycle error, got: {stderr_output}"
    );
}
