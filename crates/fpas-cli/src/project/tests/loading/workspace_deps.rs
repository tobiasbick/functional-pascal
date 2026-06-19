use super::*;
use crate::test_support::{
    create_temp_dir_under, write_library_fpasprj, write_program_fpasprj_with_workspace_deps,
};

#[test]
fn program_resolves_workspace_dependency_by_project_name() {
    let dir = create_temp_dir("workspace-dep-by-name");
    let workspace_file = dir.join("suite.fpasworkspace");
    let lib_project = dir.join("libs/greet.fpasprj");
    let app_project = dir.join("apps/hello.fpasprj");

    write_text(
        &workspace_file,
        r#"[workspace]
name = "suite"
members = ["libs/greet.fpasprj", "apps/hello.fpasprj"]
"#,
    );
    write_text(
        &lib_project,
        r#"[project]
name = "greet"
kind = "library"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    write_text(
        &lib_project.parent().unwrap().join("src/greet.fpas"),
        "unit Demo.Greet;\nconst Message: string := 'hi';\n",
    );

    write_program_fpasprj_with_workspace_deps(
        &app_project,
        "src/main.fpas",
        &["src/**/*.fpas"],
        &["greet"],
    );
    write_text(
        &app_project.parent().unwrap().join("src/main.fpas"),
        "program Hello;\nuses Demo.Greet, Std.Console;\nbegin\n  WriteLn(Message)\nend.\n",
    );

    let loaded = load_project_ok(&app_project);
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert_eq!(loaded.source_files.len(), 1);
}

#[test]
fn workspace_dependency_without_enclosing_workspace_is_rejected() {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("fpas-cli crate must be inside the repository crates directory");
    let dir = create_temp_dir_under(
        &repository_root.join("target/test-temp"),
        "workspace-dep-no-workspace",
    );
    let app_project = dir.join("app.fpasprj");

    write_program_fpasprj_with_workspace_deps(
        &app_project,
        "src/main.fpas",
        &["src/**/*.fpas"],
        &["greet"],
    );
    write_text(&dir.join("src/main.fpas"), "program App;\nbegin\nend.\n");

    let error = load_project_error(&app_project, "workspace dep requires workspace");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(
        error.contains("enclosing `.fpasworkspace`"),
        "expected workspace error, got: {error}"
    );
}

#[test]
fn unknown_workspace_dependency_name_is_rejected() {
    let dir = create_temp_dir("workspace-dep-unknown");
    let workspace_file = dir.join("suite.fpasworkspace");
    let lib_project = dir.join("lib.fpasprj");
    let app_project = dir.join("app.fpasprj");

    write_text(
        &workspace_file,
        r#"[workspace]
name = "suite"
members = ["lib.fpasprj", "app.fpasprj"]
"#,
    );
    write_library_fpasprj(&lib_project, &["src/**/*.fpas"]);
    write_text(
        &lib_project.parent().unwrap().join("src/lib.fpas"),
        "unit Lib.Core;\n",
    );

    write_program_fpasprj_with_workspace_deps(
        &app_project,
        "src/main.fpas",
        &["src/**/*.fpas"],
        &["missing-lib"],
    );
    write_text(
        &app_project.parent().unwrap().join("src/main.fpas"),
        "program App;\nbegin\nend.\n",
    );

    let error = load_project_error(&app_project, "unknown workspace dep must fail");
    fs::remove_dir_all(&dir).expect("temp directory must be removed");
    assert!(
        error.contains("Unknown workspace dependency `missing-lib`"),
        "expected unknown workspace dependency error, got: {error}"
    );
}
