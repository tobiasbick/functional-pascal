use super::*;

#[test]
fn run_cli_executes_program_project_main_file() {
    let cwd = create_temp_dir("run-program-project");
    let project_file = cwd.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
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
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/*.fpas"]
"#,
    );
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
fn run_cli_resolves_include_directive_inside_project_main() {
    let cwd = create_temp_dir("run-project-include-main");
    let project_file = cwd.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/*.fpas"]
"#,
    );
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses Std.Console;\n{$INCLUDE parts/message.inc}\nbegin\n  WriteLn(Message)\nend.\n",
    );
    write_text(
        &cwd.join("src/parts/message.inc"),
        "const Message: string := 'Hello from include';\n",
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "Hello from include\n");
    assert!(stderr_output.is_empty());
}

#[test]
fn run_cli_applies_define_flags_to_single_file_compilation() {
    let cwd = create_temp_dir("run-cli-define-flag");
    let source_path = cwd.join("main.fpas");
    write_text(
        &source_path,
        "program Main;\nuses Std.Console;\nbegin\n  {$IFDEF DEBUG}\n  WriteLn('debug');\n  {$ELSE}\n  WriteLn('release');\n  {$ENDIF}\nend.\n",
    );

    let args = vec![
        String::from("-DDEBUG"),
        source_path.to_string_lossy().to_string(),
    ];
    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_args_and_capture_output(&args, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "debug\n");
    assert!(stderr_output.is_empty());
}
