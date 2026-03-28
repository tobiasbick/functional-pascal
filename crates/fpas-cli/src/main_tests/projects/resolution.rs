use super::*;

#[test]
fn run_cli_resolves_nested_project_main_entry() {
    let cwd = create_temp_dir("run-nested-project-main-entry");
    let project_file = cwd.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "program"
main = "app/main.fpas"

[sources]
include = ["app/**/*.fpas"]
"#,
    );
    write_text(&cwd.join("app/main.fpas"), "program Main;\nbegin\nend.\n");

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0);
    assert!(stdout_output.is_empty());
    assert!(stderr_output.is_empty());
}
