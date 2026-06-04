use super::*;
use crate::test_support::{
    write_library_fpasprj, write_program_fpasprj_with_deps,
};

fn toml_path(path: &std::path::Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[test]
fn check_cli_validates_library_project() {
    let cwd = create_temp_dir("check-library");
    let project_file = cwd.join("lib.fpasprj");
    write_library_fpasprj(&project_file, &["src/**/*.fpas"]);
    write_text(
        &cwd.join("src/math.fpas"),
        "unit Lib.Math;\nfunction Double(X: integer): integer;\nbegin\n  return X + X\nend;\n",
    );

    let (exit_code, _, stderr_output) = support::run_cli_args_and_capture_output(
        &[
            String::from("check"),
            project_file.to_string_lossy().to_string(),
        ],
        &cwd,
    );
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert!(stderr_output.is_empty());
}

#[test]
fn run_cli_still_rejects_library_projects() {
    let cwd = create_temp_dir("run-library-after-check");
    let project_file = cwd.join("lib.fpasprj");
    write_library_fpasprj(&project_file, &["src/**/*.fpas"]);
    write_text(&cwd.join("src/util.fpas"), "unit Lib.Util;\n");

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(stderr_output.contains("Library projects are not executable"));
}

#[test]
fn check_cli_validates_workspace_members() {
    let cwd = create_temp_dir("check-workspace");
    let workspace_file = cwd.join("suite.fpasworkspace");
    let lib_project = cwd.join("libs/math.fpasprj");
    let app_project = cwd.join("apps/demo.fpasprj");

    write_text(
        &workspace_file,
        r#"[workspace]
name = "suite"
members = ["libs/math.fpasprj", "apps/demo.fpasprj"]
"#,
    );
    write_library_fpasprj(&lib_project, &["src/**/*.fpas"]);
    write_text(
        &lib_project.parent().unwrap().join("src/math.fpas"),
        "unit Suite.Math;\nfunction Square(X: integer): integer;\nbegin\n  return X * X\nend;\n",
    );
    let lib_dep = toml_path(&lib_project);
    write_program_fpasprj_with_deps(
        &app_project,
        "src/main.fpas",
        &["src/**/*.fpas"],
        &[lib_dep.as_str()],
    );
    write_text(
        &app_project.parent().unwrap().join("src/main.fpas"),
        "program Demo;\nuses Suite.Math, Std.Console;\nbegin\n  WriteLn(Square(6))\nend.\n",
    );

    let (exit_code, _, stderr_output) =
        support::run_cli_args_and_capture_output(&[String::from("check")], &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert!(stderr_output.is_empty());
}
