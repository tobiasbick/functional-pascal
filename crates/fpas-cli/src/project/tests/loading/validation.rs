use super::*;

#[test]
fn main_must_be_fpas_source_file() {
    let dir = create_temp_dir("main-extension");
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "program"
main = "src/main.txt"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write_text(&dir.join("src/main.txt"), "not-pascal");
    write_text(&dir.join("src/util.fpas"), "unit App.Util;");

    let error = load_project(&project_file).expect_err("main extension must be validated");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(error.contains("`project.main` must reference a `.fpas` file"));
}

#[test]
fn main_file_must_use_program_declaration() {
    let dir = create_temp_dir("main-must-be-program");
    let project_file = dir.join("app.fpasprj");
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
    write_text(&dir.join("src/main.fpas"), "unit App.Main;");
    write_text(&dir.join("src/util.fpas"), "unit App.Util;");

    let error = load_project(&project_file).expect_err("main must be a program file");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(error.contains("must declare `program`"));
}

#[test]
fn empty_project_name_is_rejected() {
    let dir = create_temp_dir("empty-name");
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = ""
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write_text(&dir.join("src/main.fpas"), "program Main;\nbegin\nend.\n");

    let error = load_project(&project_file).expect_err("empty name must fail");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(
        error.contains("`project.name` must be a non-empty string"),
        "expected empty name error, got: {error}"
    );
}

#[test]
fn empty_project_version_is_rejected() {
    let dir = create_temp_dir("empty-version");
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
version = ""
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write_text(&dir.join("src/main.fpas"), "program Main;\nbegin\nend.\n");

    let error = load_project(&project_file).expect_err("empty version must fail");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(
        error.contains("`project.version` must be a non-empty string"),
        "expected empty version error, got: {error}"
    );
}

#[test]
fn missing_project_kind_is_rejected() {
    let dir = create_temp_dir("missing-kind");
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write_text(&dir.join("src/main.fpas"), "program Main;\nbegin\nend.\n");

    let error = load_project(&project_file).expect_err("missing kind must fail");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(
        error.contains("Invalid project file"),
        "expected TOML parse error for missing kind, got: {error}"
    );
}

#[test]
fn main_file_must_exist() {
    let dir = create_temp_dir("main-not-found");
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "program"
main = "src/nonexistent.fpas"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write_text(&dir.join("src/util.fpas"), "unit App.Util;");

    let error = load_project(&project_file).expect_err("missing main file must fail");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(
        error.contains("path does not exist"),
        "expected missing file error, got: {error}"
    );
}

#[test]
fn version_field_is_optional() {
    let dir = create_temp_dir("optional-version");
    let project_file = dir.join("app.fpasprj");
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
    write_text(&dir.join("src/main.fpas"), "program Main;\nbegin\nend.\n");

    let loaded = load_project(&project_file).expect("project without version should load");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert_eq!(loaded.kind, ProjectKind::Program);
}

#[test]
fn duplicate_unit_names_are_rejected() {
    let dir = create_temp_dir("duplicate-unit-names");
    let project_file = dir.join("app.fpasprj");
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
    write_text(&dir.join("src/main.fpas"), "program Main;\nbegin\nend.\n");
    write_text(&dir.join("src/a.fpas"), "unit App.Utils;");
    write_text(&dir.join("src/b.fpas"), "unit app.utils;");

    let error = load_project(&project_file).expect_err("duplicate unit names must fail");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(error.contains("Duplicate unit name `app.utils`"));
}

#[test]
fn whitespace_only_name_is_rejected() {
    let dir = create_temp_dir("whitespace-name");
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "   "
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write_text(&dir.join("src/main.fpas"), "program Main;\nbegin\nend.\n");

    let error = load_project(&project_file).expect_err("whitespace-only name must fail");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(
        error.contains("`project.name` must be a non-empty string"),
        "expected non-empty name error, got: {error}"
    );
}

#[test]
fn whitespace_only_version_is_rejected() {
    let dir = create_temp_dir("whitespace-version");
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
version = "   "
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write_text(&dir.join("src/main.fpas"), "program Main;\nbegin\nend.\n");

    let error = load_project(&project_file).expect_err("whitespace-only version must fail");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(
        error.contains("`project.version` must be a non-empty string"),
        "expected non-empty version error, got: {error}"
    );
}

#[test]
fn missing_project_name_is_rejected() {
    let dir = create_temp_dir("missing-name");
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write_text(&dir.join("src/main.fpas"), "program Main;\nbegin\nend.\n");

    let error = load_project(&project_file).expect_err("missing name must fail");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(
        error.contains("Invalid project file"),
        "expected TOML parse error for missing name, got: {error}"
    );
}

#[test]
fn empty_main_path_is_rejected() {
    let dir = create_temp_dir("empty-main");
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "program"
main = ""

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write_text(&dir.join("src/util.fpas"), "unit App.Util;");

    let error = load_project(&project_file).expect_err("empty main path must fail");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(
        error.contains("`project.main` must be a non-empty string"),
        "expected non-empty main error, got: {error}"
    );
}

#[test]
fn whitespace_only_main_path_is_rejected() {
    let dir = create_temp_dir("whitespace-main");
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "program"
main = "   "

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write_text(&dir.join("src/util.fpas"), "unit App.Util;");

    let error = load_project(&project_file).expect_err("whitespace-only main path must fail");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(
        error.contains("`project.main` must be a non-empty string"),
        "expected non-empty main error, got: {error}"
    );
}

#[test]
fn freeform_version_string_is_accepted() {
    let dir = create_temp_dir("freeform-version");
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
version = "v2.1.0-beta+build.42"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write_text(&dir.join("src/main.fpas"), "program Main;\nbegin\nend.\n");

    let loaded = load_project(&project_file).expect("freeform version should be accepted");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert_eq!(loaded.kind, ProjectKind::Program);
}

#[test]
fn reserved_std_root_is_rejected_for_source_units() {
    let dir = create_temp_dir("reserved-std-root");
    let project_file = dir.join("app.fpasprj");
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
    write_text(&dir.join("src/main.fpas"), "program Main;\nbegin\nend.\n");
    write_text(&dir.join("src/std_unit.fpas"), "unit Std.Helpers;");

    let error = load_project(&project_file).expect_err("reserved Std namespace must fail");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(
        error.contains("reserved for standard library units"),
        "expected reserved Std namespace error, got: {error}"
    );
}
