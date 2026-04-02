use super::*;

#[test]
fn include_pattern_must_match_at_least_one_file() {
    let dir = create_temp_dir("glob-no-match");
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

    let loaded = load_project_ok(&project_file);
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert_eq!(loaded.kind, ProjectKind::Program);
    assert!(loaded.source_files.is_empty());
}

#[test]
fn include_pattern_without_matches_fails() {
    let dir = create_temp_dir("glob-no-match-real");
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["units/**/*.fpas"]
"#,
    );
    write_text(&dir.join("src/main.fpas"), "program Main;\nbegin\nend.\n");

    let error = load_project_error(&project_file, "include glob without matches must fail");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(error.contains("matched no files"));
}

#[test]
fn explicit_include_path_must_exist() {
    let dir = create_temp_dir("explicit-missing");
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/missing.fpas"]
"#,
    );
    write_text(&dir.join("src/main.fpas"), "program Main;\nbegin\nend.\n");

    let error = load_project_error(&project_file, "missing include must fail");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(error.contains("path does not exist"));
}

#[test]
fn empty_include_array_is_rejected() {
    let dir = create_temp_dir("empty-include");
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[sources]
include = []
"#,
    );
    write_text(&dir.join("src/main.fpas"), "program Main;\nbegin\nend.\n");

    let error = load_project_error(&project_file, "empty include must fail");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(
        error.contains("`sources.include` must contain at least one entry"),
        "expected empty include error, got: {error}"
    );
}

#[test]
fn empty_include_entry_is_rejected() {
    let dir = create_temp_dir("empty-include-entry");
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[sources]
include = [""]
"#,
    );
    write_text(&dir.join("src/main.fpas"), "program Main;\nbegin\nend.\n");

    let error = load_project_error(&project_file, "empty entry must fail");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(
        error.contains("empty"),
        "expected empty entry error, got: {error}"
    );
}

#[test]
fn mixed_glob_and_explicit_includes() {
    let dir = create_temp_dir("mixed-includes");
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/math.fpas", "lib/*.fpas"]
"#,
    );
    write_text(
        &dir.join("src/main.fpas"),
        "program Main;\nuses App.Math, App.Lib;\nbegin\nend.\n",
    );
    write_text(&dir.join("src/math.fpas"), "unit App.Math;");
    write_text(&dir.join("lib/helpers.fpas"), "unit App.Lib;");

    let loaded = load_project_ok(&project_file);
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert_eq!(loaded.source_files.len(), 2);
    assert!(loaded.warnings.is_empty());
}

#[test]
fn whitespace_only_include_entry_is_rejected() {
    let dir = create_temp_dir("whitespace-include-entry");
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["   "]
"#,
    );
    write_text(&dir.join("src/main.fpas"), "program Main;\nbegin\nend.\n");

    let error = load_project_error(&project_file, "whitespace-only entry must fail");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(
        error.contains("empty"),
        "expected empty entry error, got: {error}"
    );
}

#[test]
fn explicit_include_must_be_fpas_source_file() {
    let dir = create_temp_dir("include-extension");
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/readme.md"]
"#,
    );
    write_text(&dir.join("src/main.fpas"), "program Main;\nbegin\nend.\n");
    write_text(&dir.join("src/readme.md"), "docs");

    let error = load_project_error(&project_file, "include extension must be validated");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(error.contains("`sources.include` must reference a `.fpas` file"));
}
