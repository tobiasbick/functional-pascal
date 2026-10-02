use super::*;

#[test]
fn run_cli_emits_warning_for_program_source_file_and_still_runs() {
    let cwd = create_temp_dir("run-program-source-warning");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        r#"program Main;
uses App.Util as Util; uses Std.Console as Console;
begin
  Console.WriteLn(Util.GetValue());
end program;
"#,
    );
    write_text(
        &cwd.join("src/util.fpas"),
        r#"unit App.Util;
public function GetValue(): integer;
begin
  return 42;
end function;
end unit;

"#,
    );
    write_text(
        &cwd.join("src/tool.fpas"),
        r#"program Tool;
begin null;
end program;
"#,
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "42\n");
    assert!(stderr_output.contains("warning[F5036]:"));
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
        r#"program Main;
uses App.Util as Util; uses Std.Console as Console;
begin
  Console.WriteLn(Util.GetValue());
end program;
"#,
    );
    write_text(
        &cwd.join("src/util.fpas"),
        r#"unit App.Util;
public function GetValue(): integer;
begin
  return 7;
end function;
end unit;

"#,
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "7\n");
    assert!(stderr_output.contains("warning[F5035]: Duplicate source file"));
}
