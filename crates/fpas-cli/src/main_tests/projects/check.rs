use super::*;
use crate::test_support::{
    write_library_fpasprj, write_library_fpasprj_with_deps, write_program_fpasprj,
    write_program_fpasprj_with_deps,
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
        "unit Lib.Math;\npublic function Double(X: integer): integer;\nbegin\n  return X + X\nend;\n",
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
        "unit Suite.Math;\npublic function Square(X: integer): integer;\nbegin\n  return X * X\nend;\n",
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

#[test]
fn check_cli_with_no_args_discovers_workspace_in_cwd() {
    let cwd = create_temp_dir("check-workspace-discovery");
    let workspace_file = cwd.join("suite.fpasworkspace");
    let lib_project = cwd.join("lib.fpasprj");
    let app_project = cwd.join("app.fpasprj");

    write_text(
        &workspace_file,
        r#"[workspace]
name = "suite"
members = ["lib.fpasprj", "app.fpasprj"]
"#,
    );
    write_library_fpasprj(&lib_project, &["lib.fpas"]);
    write_text(&cwd.join("lib.fpas"), "unit L.Core;\n");
    write_program_fpasprj(&app_project, "main.fpas", &["main.fpas"]);
    write_text(&cwd.join("main.fpas"), "program App;\nbegin\nend.\n");

    let (exit_code, _, stderr_output) =
        support::run_cli_args_and_capture_output(&[String::from("check")], &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert!(stderr_output.is_empty());
}

#[test]
fn check_cli_with_explicit_workspace_path_argument() {
    let cwd = create_temp_dir("check-workspace-explicit");
    let workspace_file = cwd.join("suite.fpasworkspace");
    let lib_project = cwd.join("lib.fpasprj");

    write_text(
        &workspace_file,
        r#"[workspace]
name = "suite"
members = ["lib.fpasprj"]
"#,
    );
    write_library_fpasprj(&lib_project, &["lib.fpas"]);
    write_text(&cwd.join("lib.fpas"), "unit L.Core;\n");

    let (exit_code, _, stderr_output) = support::run_cli_args_and_capture_output(
        &[
            String::from("check"),
            workspace_file.to_string_lossy().to_string(),
        ],
        &cwd,
    );
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert!(stderr_output.is_empty());
}

#[test]
fn check_cli_fails_on_type_error_in_library_project() {
    let cwd = create_temp_dir("check-library-type-error");
    let project_file = cwd.join("lib.fpasprj");
    write_library_fpasprj(&project_file, &["src/**/*.fpas"]);
    write_text(
        &cwd.join("src/math.fpas"),
        "unit Lib.Math;\npublic function Bad(X: integer): string;\nbegin\n  return X + X\nend;\n",
    );

    let (exit_code, _, stderr_output) = support::run_cli_args_and_capture_output(
        &[
            String::from("check"),
            project_file.to_string_lossy().to_string(),
        ],
        &cwd,
    );
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1, "expected type-check failure");
    assert!(
        stderr_output.contains("error"),
        "stderr should contain compile diagnostic: {stderr_output}"
    );
}

#[test]
fn check_cli_validates_transitive_library_dependencies() {
    let cwd = create_temp_dir("check-transitive-libs");
    let base_dir = cwd.join("libs/base");
    let util_dir = cwd.join("libs/util");
    let app_dir = cwd.join("apps/demo");
    let base_project = base_dir.join("base.fpasprj");
    let util_project = util_dir.join("util.fpasprj");
    let app_project = app_dir.join("demo.fpasprj");

    write_library_fpasprj(&base_project, &["src/**/*.fpas"]);
    write_text(
        &base_dir.join("src/base.fpas"),
        "unit Lib.Base;\npublic const Tag: string := 'ok';\n",
    );

    write_library_fpasprj_with_deps(&util_project, &["src/**/*.fpas"], &["../base/base.fpasprj"]);
    write_text(
        &util_dir.join("src/util.fpas"),
        "unit Lib.Util;\nuses Lib.Base;\npublic function Label(): string;\nbegin\n  return Tag\nend;\n",
    );

    let util_dep = toml_path(&util_project);
    write_program_fpasprj_with_deps(
        &app_project,
        "src/main.fpas",
        &["src/**/*.fpas"],
        &[util_dep.as_str()],
    );
    write_text(
        &app_dir.join("src/main.fpas"),
        "program Demo;\nuses Lib.Util, Std.Console;\nbegin\n  WriteLn(Label())\nend.\n",
    );

    let (exit_code, _, stderr_output) = support::run_cli_args_and_capture_output(
        &[
            String::from("check"),
            app_project.to_string_lossy().to_string(),
        ],
        &app_dir,
    );
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert!(stderr_output.is_empty());
}

#[test]
fn check_cli_validates_directory_of_sources() {
    let cwd = create_temp_dir("check-source-directory");
    write_text(&cwd.join("ok.fpas"), "program Ok;\nbegin\nend.\n");
    write_text(
        &cwd.join("bad.fpas"),
        "program Bad;\nuses Std.Console;\nbegin\n  WriteLn(1 + 'nope')\nend.\n",
    );

    let (exit_code, _, stderr_output) = support::run_cli_args_and_capture_output(
        &[String::from("check"), cwd.to_string_lossy().to_string()],
        &cwd,
    );
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("error"),
        "stderr should contain compile diagnostic: {stderr_output}"
    );
}

#[test]
fn check_cli_validates_directory_program_with_sibling_unit_without_sidecars() {
    let cwd = create_temp_dir("check-directory-shared-unit");
    write_text(
        &cwd.join("math.fpas"),
        "unit Demo.Math;\npublic function Answer(): integer;\nbegin\n  return 42\nend;\n",
    );
    write_text(
        &cwd.join("main.fpas"),
        "program Main;\nuses Demo.Math;\nbegin\n  Answer()\nend.\n",
    );

    let (exit_code, _, stderr_output) = support::run_cli_args_and_capture_output(
        &[String::from("check"), cwd.to_string_lossy().to_string()],
        &cwd,
    );
    let sidecar_exists = cwd.join("math.fpascu").exists();
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert!(stderr_output.is_empty());
    assert!(
        !sidecar_exists,
        "directory checks must not publish sidecars"
    );
}

#[test]
fn check_cli_validates_directory_containing_only_units() {
    let cwd = create_temp_dir("check-directory-units-only");
    write_text(
        &cwd.join("base.fpas"),
        "unit Demo.Base;\npublic const Answer: integer := 42;\n",
    );
    write_text(
        &cwd.join("derived.fpas"),
        "unit Demo.Derived;\nuses Demo.Base;\npublic function Value(): integer;\nbegin\n  return Answer\nend;\n",
    );

    let (exit_code, _, stderr_output) = support::run_cli_args_and_capture_output(
        &[String::from("check"), cwd.to_string_lossy().to_string()],
        &cwd,
    );
    let sidecars_exist = cwd.join("base.fpascu").exists() || cwd.join("derived.fpascu").exists();
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert!(stderr_output.is_empty());
    assert!(
        !sidecars_exist,
        "directory checks must not publish sidecars"
    );
}

#[test]
fn check_cli_validates_multiple_programs_against_shared_units() {
    let cwd = create_temp_dir("check-directory-multiple-programs");
    write_text(
        &cwd.join("shared.fpas"),
        "unit Demo.Shared;\npublic function Value(): integer;\nbegin\n  return 7\nend;\n",
    );
    write_text(
        &cwd.join("first.fpas"),
        "program First;\nuses Demo.Shared;\nbegin\n  Value()\nend.\n",
    );
    write_text(
        &cwd.join("second.fpas"),
        "program Second;\nuses Demo.Shared;\nbegin\n  Value()\nend.\n",
    );

    let (exit_code, _, stderr_output) = support::run_cli_args_and_capture_output(
        &[String::from("check"), cwd.to_string_lossy().to_string()],
        &cwd,
    );
    let sidecar_exists = cwd.join("shared.fpascu").exists();
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert!(stderr_output.is_empty());
    assert!(
        !sidecar_exists,
        "directory checks must not publish sidecars"
    );
}
