use super::*;

#[test]
fn run_cli_rejects_library_projects() {
    let cwd = create_temp_dir("run-library-project");
    let project_file = cwd.join("lib.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "lib"
kind = "library"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write_text(&cwd.join("src/util.fpas"), "unit Lib.Util;");

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(stderr_output.contains("Library projects are not executable"));
    assert!(!stderr_output.contains("warning:"));
}

#[test]
fn run_cli_reports_cyclic_unit_dependencies() {
    let cwd = create_temp_dir("run-cycle");
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
        "program Main;\nuses App.A;\nbegin\nend.\n",
    );
    write_text(&cwd.join("src/a.fpas"), "unit App.A;\nuses App.B;\n");
    write_text(&cwd.join("src/b.fpas"), "unit App.B;\nuses App.A;\n");

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(stderr_output.contains("Cyclic unit dependency detected"));
}

#[test]
fn run_cli_reports_unknown_user_unit() {
    let cwd = create_temp_dir("run-unknown-unit");
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
        "program Main;\nuses App.Missing;\nbegin\nend.\n",
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(stderr_output.contains("Unknown unit `App.Missing`"));
}

#[test]
fn run_cli_reports_ambiguous_user_imports() {
    let cwd = create_temp_dir("run-ambiguous-import");
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
        "program Main;\nuses App.Math, App.Advanced;\nbegin\n  Add(1, 2)\nend.\n",
    );
    write_text(
        &cwd.join("src/math.fpas"),
        "unit App.Math;\nfunction Add(A: integer; B: integer): integer;\nbegin\n  return A + B\nend;\n",
    );
    write_text(
        &cwd.join("src/advanced.fpas"),
        "unit App.Advanced;\nfunction Add(A: integer; B: integer): integer;\nbegin\n  return A - B\nend;\n",
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(stderr_output.contains("Ambiguous imported symbol `Add`"));
}
