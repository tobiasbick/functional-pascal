use super::*;

#[test]
fn malformed_toml_is_rejected() {
    let dir = create_temp_dir("malformed-toml");
    let project_file = dir.join("app.fpasprj");
    write_text(&project_file, "this is not valid TOML {{{");

    let error = load_project(&project_file).expect_err("malformed TOML must fail");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(
        error.contains("Invalid project file"),
        "expected TOML parse error, got: {error}"
    );
}

#[test]
fn missing_project_section_is_rejected() {
    let dir = create_temp_dir("missing-project-section");
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[sources]
include = ["src/**/*.fpas"]
"#,
    );

    let error = load_project(&project_file).expect_err("missing [project] must fail");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(
        error.contains("Invalid project file"),
        "expected section error, got: {error}"
    );
}

#[test]
fn missing_sources_section_is_rejected() {
    let dir = create_temp_dir("missing-sources-section");
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"
"#,
    );

    let error = load_project(&project_file).expect_err("missing [sources] must fail");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(
        error.contains("Missing `[sources]` section"),
        "expected missing sources error, got: {error}"
    );
}
