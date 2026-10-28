use super::*;

#[test]
fn environment_report_describes_the_running_toolchain() {
    let cwd = create_temp_dir("toolchain-env");
    let (exit_code, stdout, stderr) = support::run_cli_args_and_capture_output(
        &[String::from("env"), String::from("--json")],
        &cwd,
    );
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "environment query failed: {stderr}");
    assert!(stderr.is_empty());
    let report: serde_json::Value =
        serde_json::from_str(&stdout).expect("environment output must be JSON");
    assert_eq!(report["schemaVersion"], 1);
    assert_eq!(report["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(
        std::path::Path::new(
            report["languageServer"]
                .as_str()
                .expect("language-server path")
        ),
        std::path::Path::new(report["executable"].as_str().expect("executable path"))
            .with_file_name(format!("fpas-lsp{}", std::env::consts::EXE_SUFFIX))
    );
    assert!(
        std::path::Path::new(report["executable"].as_str().expect("executable path")).is_absolute()
    );
    assert!(
        std::path::Path::new(
            report["standardLibrary"]
                .as_str()
                .expect("standard-library path")
        )
        .join("stdlib.fpasprj")
        .is_file()
    );
}

#[test]
fn toolchain_commands_have_focused_argument_contracts() {
    let cwd = create_temp_dir("toolchain-arguments");

    assert_eq!(
        resolve_cli_config(&[String::from("env"), String::from("--json")], &cwd),
        Ok(ResolvedCli::Environment)
    );
    assert_eq!(
        resolve_cli_config(&[String::from("env"), String::from("--help")], &cwd),
        Ok(ResolvedCli::Help(crate::cli_input::HelpTopic::Env))
    );
    assert!(
        resolve_cli_config(&[String::from("env")], &cwd)
            .expect_err("env without --json must fail")
            .contains("fpas env --json")
    );

    fs::remove_dir_all(&cwd).expect("temp directory must be removed");
}
