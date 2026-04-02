use super::*;

#[test]
fn duplicate_sources_are_ignored_with_warning() {
    let dir = create_temp_dir("duplicate-source");
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/util.fpas", "src/*.fpas", "src/util.fpas"]
"#,
    );
    write_text(&dir.join("src/main.fpas"), "program Main;\nbegin\nend.\n");
    write_text(&dir.join("src/util.fpas"), "unit App.Util;");

    let loaded = load_project_ok(&project_file);
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert_eq!(loaded.source_files.len(), 1);
    assert!(
        loaded
            .warnings
            .iter()
            .any(|warning| warning.contains("Duplicate source file"))
    );
}

#[test]
fn main_file_is_excluded_from_sources_when_matched_by_glob() {
    let dir = create_temp_dir("main-excluded-from-glob");
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
    write_text(&dir.join("src/util.fpas"), "unit App.Util;");

    let loaded = load_project_ok(&project_file);
    fs::remove_dir_all(&dir).expect("temp directory must be removed");

    // main.fpas should be excluded even though *.fpas matches it
    assert_eq!(loaded.source_files.len(), 1);
    assert!(
        loaded.source_files[0].ends_with("util.fpas"),
        "only non-main source should remain"
    );
    assert!(loaded.main.is_some());
    assert!(loaded.warnings.is_empty());
}

#[test]
fn main_file_is_removed_when_sources_include_relative_and_absolute_main_paths() {
    let dir = create_temp_dir("main-dedup-relative-absolute");
    let main_path = dir.join("src/main.fpas");
    let main_path_text = toml_path(&main_path);
    let project_file = dir.join("app.fpasprj");
    write_text(
        &project_file,
        &format!(
            r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/main.fpas", "{main_path_text}", "src/util.fpas"]
"#
        ),
    );
    write_text(&main_path, "program Main;\nbegin\nend.\n");
    write_text(&dir.join("src/util.fpas"), "unit App.Util;");

    let loaded = load_project_ok(&project_file);
    fs::remove_dir_all(&dir).expect("temp directory must be removed");

    assert_eq!(loaded.source_files.len(), 1);
    assert!(loaded.source_files[0].ends_with("util.fpas"));
    assert!(
        loaded
            .warnings
            .iter()
            .any(|warning| warning.contains("Duplicate source file"))
    );
}

#[test]
fn glob_matching_only_main_file_yields_empty_sources() {
    let dir = create_temp_dir("glob-only-main");
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

    let loaded = load_project_ok(&project_file);
    fs::remove_dir_all(&dir).expect("temp directory must be removed");

    assert!(
        loaded.source_files.is_empty(),
        "only main was matched, so no sources remain"
    );
    assert!(loaded.main.is_some());
    assert!(loaded.warnings.is_empty());
}

#[test]
fn source_program_file_is_skipped_with_warning() {
    let dir = create_temp_dir("skip-source-program");
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
    write_text(&dir.join("src/util.fpas"), "unit App.Util;");
    write_text(&dir.join("src/tool.fpas"), "program Tool;\nbegin\nend.\n");

    let loaded = load_project_ok(&project_file);
    fs::remove_dir_all(&dir).expect("temp directory must be removed");

    assert_eq!(loaded.source_files.len(), 1);
    assert!(loaded.source_files[0].ends_with("util.fpas"));
    assert!(
        loaded
            .warnings
            .iter()
            .any(|warning| warning.contains("declares `program Tool` and was skipped"))
    );
}

#[test]
fn duplicate_sources_are_ignored_across_multiple_path_spellings() {
    let dir = create_temp_dir("duplicate-source-spellings");
    let util_path = dir.join("src/util.fpas");
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
include = ["src/util.fpas", "./src/util.fpas", "src/../src/util.fpas", "{util_path_text}"]
"#
        ),
    );
    write_text(&dir.join("src/main.fpas"), "program Main;\nbegin\nend.\n");
    write_text(&util_path, "unit App.Util;");

    let loaded = load_project_ok(&project_file);
    fs::remove_dir_all(&dir).expect("temp directory must be removed");

    assert_eq!(loaded.source_files.len(), 1);
    assert!(loaded.source_files[0].ends_with("util.fpas"));
    assert_eq!(
        loaded
            .warnings
            .iter()
            .filter(|warning| warning.contains("Duplicate source file"))
            .count(),
        3
    );
}
