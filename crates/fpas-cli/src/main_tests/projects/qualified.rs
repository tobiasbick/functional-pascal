use super::*;

#[test]
fn qualified_const_access_from_user_unit() {
    let cwd = create_temp_dir("run-qual-const");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses App.Config as Config, Std.Console as Console;\nbegin\n  Console.WriteLn(Config.MaxVal);\nend.\n",
    );
    write_text(
        &cwd.join("src/config.fpas"),
        "unit App.Config;\n\npublic const\n  MaxVal: integer := 256;\nend unit;\n",
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "256\n");
}

#[test]
fn case_insensitive_qualified_call() {
    let cwd = create_temp_dir("run-case-qual-call");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses App.Lib as Lib, Std.Console as Console;\nbegin\n  Console.WriteLn(Lib.GetValue());\nend.\n",
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        "unit App.Lib;\npublic function GetValue(): integer;\nbegin\n  return 44;\nend function;\nend unit;\n",
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "44\n");
}

#[test]
fn qualified_name_call_to_user_unit_function() {
    let cwd = create_temp_dir("run-qualified-call");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses App.Lib as Lib, Std.Console as Console;\nbegin\n  Console.WriteLn(Lib.GetValue());\nend.\n",
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        "unit App.Lib;\npublic function GetValue(): integer;\nbegin\n  return 77;\nend function;\nend unit;\n",
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "77\n");
}

#[test]
fn mixed_short_and_qualified_calls_in_same_program() {
    let cwd = create_temp_dir("run-mixed-calls");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses App.Lib as Lib, Std.Console as Console;\nbegin\n  Console.WriteLn(Lib.GetValue());\n  Console.WriteLn(Lib.GetValue());\nend.\n",
    );
    write_text(
        &cwd.join("src/lib.fpas"),
        "unit App.Lib;\npublic function GetValue(): integer;\nbegin\n  return 55;\nend function;\nend unit;\n",
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "55\n55\n");
}

#[test]
fn deep_transitive_chain_four_levels() {
    let cwd = create_temp_dir("run-deep-transitive");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses App.A as A, Std.Console as Console;\nbegin\n  Console.WriteLn(A.CallA());\nend.\n",
    );
    write_text(
        &cwd.join("src/a.fpas"),
        "unit App.A;\nuses App.B as B;\npublic function CallA(): integer;\nbegin\n  return B.CallB() + 1;\nend function;\nend unit;\n",
    );
    write_text(
        &cwd.join("src/b.fpas"),
        "unit App.B;\nuses App.C as C;\npublic function CallB(): integer;\nbegin\n  return C.CallC() + 10;\nend function;\nend unit;\n",
    );
    write_text(
        &cwd.join("src/c.fpas"),
        "unit App.C;\npublic function CallC(): integer;\nbegin\n  return 100;\nend function;\nend unit;\n",
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "111\n");
}
