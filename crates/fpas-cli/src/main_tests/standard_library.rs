use super::*;

#[test]
fn source_standard_library_is_loaded_from_explicit_override() {
    let cwd = create_temp_dir("std-version-override");
    let library = cwd.join("library");
    write_text(
        &library.join("stdlib.fpasprj"),
        r#"[project]
name = "override-standard-library"
kind = "library"

[exports]
units = ["Std.Version"]

[sources]
include = ["Std/**/*.fpas"]
"#,
    );
    write_text(
        &library.join("Std/Version.fpas"),
        "unit Std.Version;\nconst\n  CompilerVersion: string := 'override';\n",
    );
    let program = cwd.join("main.fpas");
    write_text(
        &program,
        "program Main;\nuses Std.Console, Std.Version;\nbegin\n  WriteLn(CompilerVersion)\nend.\n",
    );

    let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(
        &[
            String::from("run"),
            String::from("--std-lib"),
            library.to_string_lossy().into_owned(),
            program.to_string_lossy().into_owned(),
        ],
        &cwd,
    );
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit, 0, "stderr: {stderr}");
    assert_eq!(stdout, "override\n");
}

#[test]
fn source_standard_library_is_loaded_for_test_runs() {
    let cwd = create_temp_dir("std-version-test-override");
    let library = cwd.join("library");
    write_text(
        &library.join("stdlib.fpasprj"),
        r#"[project]
name = "override-standard-library"
kind = "library"

[exports]
units = ["Std.Version"]

[sources]
include = ["Std/**/*.fpas"]
"#,
    );
    write_text(
        &library.join("Std/Version.fpas"),
        "unit Std.Version;\nconst\n  LibraryVersion: string := 'test-override';\n",
    );
    let test = cwd.join("version_test.fpas");
    write_text(
        &test,
        "program VersionTest;\nuses Std.Test, Std.Version;\nbegin\n  AssertEquals('test-override', LibraryVersion)\nend.\n",
    );

    let (exit, _stdout, stderr) = support::run_cli_args_and_capture_output(
        &[
            String::from("test"),
            String::from("--std-lib"),
            library.to_string_lossy().into_owned(),
            test.to_string_lossy().into_owned(),
        ],
        &cwd,
    );
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit, 0, "stderr: {stderr}");
}

#[test]
fn source_standard_library_is_copied_beside_the_cli_binary() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root");
    let program = root.join("target/test-std-version.fpas");
    write_text(
        &program,
        "program Main;\nuses Std.Console, Std.Version;\nbegin\n  WriteLn(LibraryVersion)\nend.\n",
    );

    let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(
        &[String::from("run"), program.to_string_lossy().into_owned()],
        root,
    );
    fs::remove_file(&program).expect("temporary program must be removed");

    assert_eq!(exit, 0, "stderr: {stderr}");
    assert_eq!(stdout, "0.0.1\n");
}

fn run_repo_std_program(rel_path: &str) -> (i32, String, String) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root");
    let program = root.join(rel_path);
    support::run_cli_args_and_capture_output(
        &[
            String::from("run"),
            String::from("--std-lib"),
            root.join("lib").to_string_lossy().into_owned(),
            program.to_string_lossy().into_owned(),
        ],
        root,
    )
}

#[test]
fn tui_rejects_duplicate_control_ids() {
    let (exit, _stdout, stderr) =
        run_repo_std_program("tests/stdlib/tui/duplicate_control_id_runtime_error.fpas");

    assert_ne!(exit, 0, "duplicate Tui control ids must fail");
    assert!(
        stderr.contains("Tui control id must be unique in one tree: 1"),
        "stderr: {stderr}"
    );
}

#[test]
fn tui_rejects_forged_non_positive_element_control_ids() {
    let (exit, _stdout, stderr) =
        run_repo_std_program("tests/stdlib/tui/invalid_element_control_id_runtime_error.fpas");

    assert_ne!(exit, 0, "non-positive Tui element control ids must fail");
    assert!(
        stderr.contains("Tui interactive elements require a positive control id"),
        "stderr: {stderr}"
    );
}

#[test]
fn tui_rejects_forged_non_positive_element_action_ids() {
    let (exit, _stdout, stderr) =
        run_repo_std_program("tests/stdlib/tui/invalid_element_action_id_runtime_error.fpas");

    assert_ne!(exit, 0, "non-positive Tui element action ids must fail");
    assert!(
        stderr.contains("Tui interactive elements require a positive action id"),
        "stderr: {stderr}"
    );
}

#[test]
fn tui_rejects_invalid_cell_glyphs() {
    let (exit, _stdout, stderr) =
        run_repo_std_program("tests/stdlib/tui/invalid_cell_glyph_runtime_error.fpas");

    assert_ne!(exit, 0, "empty Tui cell glyphs must fail");
    assert!(
        stderr.contains("GraphemeWidth requires one non-zero-width extended grapheme cluster"),
        "stderr: {stderr}"
    );
}
