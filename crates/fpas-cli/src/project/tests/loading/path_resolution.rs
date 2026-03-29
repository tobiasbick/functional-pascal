use super::*;

fn toml_path(path: &std::path::Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[test]
fn program_project_accepts_absolute_main_path() {
    let dir = create_temp_dir("absolute-main");
    let main_path = dir.join("src/main.fpas");
    let main_path_text = toml_path(&main_path);
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        &format!(
            r#"[project]
name = "app"
kind = "program"
main = "{main_path_text}"

[sources]
include = ["src/**/*.fpas"]
"#
        ),
    );
    write_text(&main_path, "program Main;\nbegin\nend.\n");

    let loaded = load_project(&project_file).expect("absolute main path should load");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");

    assert_eq!(loaded.main, Some(main_path));
    assert!(loaded.source_files.is_empty());
}

#[test]
fn sources_include_accepts_absolute_file_path() {
    let dir = create_temp_dir("absolute-include");
    let util_path = dir.join("shared/util.fpas");
    let util_path_text = toml_path(&util_path);
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        &format!(
            r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["{util_path_text}"]
"#
        ),
    );
    write_text(&dir.join("src/main.fpas"), "program Main;\nbegin\nend.\n");
    write_text(&util_path, "unit App.Util;");

    let loaded = load_project(&project_file).expect("absolute include path should load");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");

    assert_eq!(loaded.source_files, vec![util_path]);
    assert!(loaded.warnings.is_empty());
}

#[test]
fn main_path_must_point_to_a_file() {
    let dir = create_temp_dir("main-directory");
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "program"
main = "src"

[sources]
include = ["src/util.fpas"]
"#,
    );
    write_text(&dir.join("src/util.fpas"), "unit App.Util;");

    let error = load_project(&project_file).expect_err("main directory must fail");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");

    assert!(
        error.contains("`project.main` must point to a file"),
        "expected file path error, got: {error}"
    );
}

#[test]
fn explicit_include_path_must_point_to_a_file() {
    let dir = create_temp_dir("include-directory");
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src"]
"#,
    );
    write_text(&dir.join("src/main.fpas"), "program Main;\nbegin\nend.\n");

    let error = load_project(&project_file).expect_err("include directory must fail");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");

    assert!(
        error.contains("`sources.include` must point to a file"),
        "expected file path error, got: {error}"
    );
}

#[test]
fn include_glob_rejects_non_source_files() {
    let dir = create_temp_dir("glob-non-source");
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/**/*"]
"#,
    );
    write_text(&dir.join("src/main.fpas"), "program Main;\nbegin\nend.\n");
    write_text(&dir.join("src/readme.md"), "not pascal");

    let error = load_project(&project_file).expect_err("glob with non-source file must fail");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");

    assert!(
        error.contains("matched a non-source file"),
        "expected non-source glob error, got: {error}"
    );
}
