use super::*;

#[test]
fn run_cli_help_and_version_exit_zero() {
    let cwd = create_temp_dir("run-help");

    let (code_h, help_text, stderr_h) =
        run_cli_args_and_capture_output(&[String::from("--help")], &cwd);
    assert_eq!(code_h, 0);
    assert!(help_text.contains("Usage:"));
    assert!(help_text.contains("fpas run"));
    assert!(help_text.contains("Examples:"));
    assert!(stderr_h.is_empty());

    let (code_bare, bare_help, stderr_bare) = run_cli_args_and_capture_output(&[], &cwd);
    assert_eq!(code_bare, 0);
    assert!(bare_help.contains("fpas run"));
    assert!(stderr_bare.is_empty());

    let (code_v, ver, stderr_v) =
        run_cli_args_and_capture_output(&[String::from("--version")], &cwd);
    assert_eq!(code_v, 0);
    assert!(ver.starts_with("fpas "));
    assert!(stderr_v.is_empty());

    fs::remove_dir_all(&cwd).expect("temp directory must be removed");
}

#[test]
fn run_cli_subcommand_help_is_focused_and_includes_examples() {
    let cwd = create_temp_dir("subcommand-help");

    for (command, expected_usage, excluded_usage) in [
        ("build", "fpas build [--std-lib", "fpas test ["),
        ("run", "fpas run [--std-lib", "fpas test ["),
        ("check", "fpas check [--std-lib", "fpas test ["),
        ("fmt", "fpas fmt [<path>...]", "fpas test ["),
        ("test", "fpas test [--std-lib", "fpas fmt --stdout"),
    ] {
        let (exit_code, stdout, stderr) =
            run_cli_args_and_capture_output(&[String::from(command), String::from("--help")], &cwd);

        assert_eq!(exit_code, 0, "{command} help must succeed: {stderr}");
        assert!(
            stdout.contains(expected_usage),
            "unexpected {command} help: {stdout}"
        );
        assert!(
            stdout.contains("Examples:"),
            "{command} help needs examples"
        );
        assert!(
            !stdout.contains(excluded_usage),
            "{command} help must not include unrelated command details: {stdout}"
        );
        assert!(stderr.is_empty());
    }

    fs::remove_dir_all(&cwd).expect("temp directory must be removed");
}
