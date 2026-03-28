use super::*;

#[test]
fn resolve_cli_input_uses_explicit_source_file() {
    let cwd = create_temp_dir("source");
    let result = resolve_cli_input(&[String::from("src/main.fpas")], &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(result, Ok(CliInput::SourceFile(cwd.join("src/main.fpas"))));
}

#[test]
fn resolve_cli_input_uses_explicit_project_file() {
    let cwd = create_temp_dir("project");
    let result = resolve_cli_input(&[String::from("my-app.fpasprj")], &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(
        result,
        Ok(CliInput::ProjectFile(cwd.join("my-app.fpasprj")))
    );
}

#[test]
fn resolve_cli_input_rejects_unknown_extension() {
    let cwd = create_temp_dir("unknown-ext");
    let result = resolve_cli_input(&[String::from("project.toml")], &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    let error = result.expect_err("unknown extension must fail");
    assert!(error.contains("Unsupported input"));
    assert!(error.contains(".fpas"));
    assert!(error.contains(".fpasprj"));
}

#[test]
fn resolve_cli_input_discovers_project_file_when_no_args_are_given() {
    let cwd = create_temp_dir("discover-one");
    let project_path = cwd.join("demo.fpasprj");
    write_file(&project_path);

    let result = resolve_cli_input(&[], &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(result, Ok(CliInput::ProjectFile(project_path)));
}

#[test]
fn resolve_cli_input_fails_when_no_project_file_exists() {
    let cwd = create_temp_dir("discover-none");
    let result = resolve_cli_input(&[], &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    let error = result.expect_err("missing project file must fail");
    assert!(error.contains("No `.fpasprj` file found"));
}

#[test]
fn resolve_cli_input_fails_when_multiple_project_files_exist() {
    let cwd = create_temp_dir("discover-many");
    write_file(&cwd.join("a.fpasprj"));
    write_file(&cwd.join("b.fpasprj"));

    let result = resolve_cli_input(&[], &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    let error = result.expect_err("multiple project files must fail");
    assert!(error.contains("Found multiple `.fpasprj` files"));
    assert!(error.contains("a.fpasprj"));
    assert!(error.contains("b.fpasprj"));
}

#[test]
fn resolve_cli_input_handles_case_insensitive_extensions() {
    let cwd = create_temp_dir("case-ext");
    let result_fpas = resolve_cli_input(&[String::from("Main.FPAS")], &cwd);
    let result_prj = resolve_cli_input(&[String::from("app.FPASPRJ")], &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(result_fpas, Ok(CliInput::SourceFile(cwd.join("Main.FPAS"))));
    assert_eq!(
        result_prj,
        Ok(CliInput::ProjectFile(cwd.join("app.FPASPRJ")))
    );
}

#[test]
fn resolve_cli_input_rejects_more_than_one_argument() {
    let cwd = create_temp_dir("too-many-args");
    let result = resolve_cli_input(&[String::from("a.fpas"), String::from("b.fpas")], &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    let error = result.expect_err("multiple arguments must fail");
    assert_eq!(error, "Usage: fpas [<file.fpas | file.fpasprj>]");
}
