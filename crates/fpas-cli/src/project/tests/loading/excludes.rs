use super::*;

#[test]
fn exclude_pattern_removes_matching_sources() {
    let dir = create_temp_dir("exclude-glob");
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]
exclude = ["src/generated/**/*.fpas"]
"#,
    );
    write_text(&dir.join("src/main.fpas"), "program Main;\nbegin\nend.\n");
    write_text(
        &dir.join("src/generated/stub.fpas"),
        "unit App.Generated;\n",
    );
    write_text(&dir.join("src/util.fpas"), "unit App.Util;\n");

    let loaded = load_project_ok(&project_file);
    fs::remove_dir_all(&dir).expect("temp directory must be removed");

    assert_eq!(loaded.source_files.len(), 1);
    assert!(
        loaded
            .source_files
            .iter()
            .all(|path| path.file_name().is_some_and(|name| name == "util.fpas")),
        "expected only util.fpas, got: {:?}",
        loaded.source_files
    );
}

#[test]
fn exclude_glob_with_no_matches_is_allowed() {
    let dir = create_temp_dir("exclude-no-match");
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]
exclude = ["src/missing/**/*.fpas"]
"#,
    );
    write_text(&dir.join("src/main.fpas"), "program Main;\nbegin\nend.\n");
    write_text(&dir.join("src/util.fpas"), "unit App.Util;\n");

    let loaded = load_project_ok(&project_file);
    fs::remove_dir_all(&dir).expect("temp directory must be removed");

    assert_eq!(loaded.source_files.len(), 1);
}

#[test]
fn empty_exclude_entry_is_rejected() {
    let dir = create_temp_dir("exclude-empty-entry");
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]
exclude = [""]
"#,
    );
    write_text(&dir.join("src/main.fpas"), "program Main;\nbegin\nend.\n");

    let error = load_project_error(&project_file, "empty exclude entry must fail");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(
        error.contains("sources.exclude"),
        "expected exclude field error, got: {error}"
    );
}
