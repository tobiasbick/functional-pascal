use super::*;

#[test]
fn program_project_requires_main() {
    let dir = create_temp_dir("program-main-required");
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "program"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write_text(&dir.join("src/util.fpas"), "unit App.Util;");

    let error = load_project(&project_file).expect_err("program projects need main");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(error.contains("Program projects require `project.main`"));
}

#[test]
fn library_project_rejects_main() {
    let dir = create_temp_dir("library-main-forbidden");
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "lib"
kind = "library"
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write_text(&dir.join("src/util.fpas"), "unit Lib.Util;");

    let error = load_project(&project_file).expect_err("library project must reject main");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(error.contains("must not define `project.main`"));
}

#[test]
fn invalid_project_kind_is_rejected() {
    let dir = create_temp_dir("invalid-kind");
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "executable"
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write_text(&dir.join("src/main.fpas"), "program Main;\nbegin\nend.\n");

    let error = load_project(&project_file).expect_err("invalid kind must fail");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(
        error.contains("Invalid `project.kind`"),
        "expected invalid kind error, got: {error}"
    );
}

#[test]
fn empty_project_kind_is_rejected() {
    let dir = create_temp_dir("empty-kind");
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = ""
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write_text(&dir.join("src/main.fpas"), "program Main;\nbegin\nend.\n");

    let error = load_project(&project_file).expect_err("empty kind must fail");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(
        error.contains("Invalid `project.kind`"),
        "expected invalid kind error, got: {error}"
    );
}

#[test]
fn whitespace_only_project_kind_is_rejected() {
    let dir = create_temp_dir("whitespace-kind");
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "   "
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write_text(&dir.join("src/main.fpas"), "program Main;\nbegin\nend.\n");

    let error = load_project(&project_file).expect_err("whitespace-only kind must fail");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(
        error.contains("Invalid `project.kind`"),
        "expected invalid kind error, got: {error}"
    );
}

#[test]
fn library_without_sources_section_is_rejected() {
    let dir = create_temp_dir("library-no-sources");
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "lib"
kind = "library"
"#,
    );

    let error = load_project(&project_file).expect_err("library without sources must fail");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(
        error.contains("Missing `[sources]` section"),
        "expected missing sources error, got: {error}"
    );
}

#[test]
fn library_project_loads_without_main() {
    let dir = create_temp_dir("library-valid");
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "lib"
kind = "library"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write_text(&dir.join("src/util.fpas"), "unit Lib.Util;");

    let loaded = load_project(&project_file).expect("library project should load");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert_eq!(loaded.kind, ProjectKind::Library);
    assert!(loaded.main.is_none());
    assert_eq!(loaded.source_files.len(), 1);
    assert!(loaded.warnings.is_empty());
}
