use super::*;

const TEST_PROJECT: &str =
    "[project]\nname = \"demo\"\nkind = \"test\"\n\n[sources]\ninclude = [\"*.fpas\"]\n";

#[test]
fn resolve_cli_config_parses_native_application_options() {
    let cwd = create_temp_dir("build-native-options");
    let result = resolve_cli_config(
        &[
            String::from("build"),
            String::from("--executable"),
            String::from("--name"),
            String::from("hello"),
            String::from("app.fpasprj"),
        ],
        &cwd,
    );
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    match result {
        Ok(ResolvedCli::Build(config)) => {
            assert!(config.executable);
            assert_eq!(config.name.as_deref(), Some("hello"));
            assert_eq!(config.input, CliInput::ProjectFile(cwd.join("app.fpasprj")));
        }
        other => panic!("expected build config, got {other:?}"),
    }
}

#[test]
fn resolve_cli_config_rejects_name_without_executable() {
    let cwd = create_temp_dir("build-name-without-native");
    let result = resolve_cli_config(
        &[
            String::from("build"),
            String::from("--name"),
            String::from("hello"),
            String::from("app.fpasprj"),
        ],
        &cwd,
    );
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    let error = result.expect_err("name without executable must fail");
    assert!(error.contains("`--name` requires `--executable`"));
}

#[test]
fn resolve_cli_config_reports_build_specific_missing_stdlib_help() {
    let cwd = create_temp_dir("build-missing-stdlib");
    let result = resolve_cli_config(&[String::from("build"), String::from("--std-lib")], &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    let error = result.expect_err("missing build stdlib path must fail");
    assert!(error.contains("fpas build --std-lib ./lib my-app.fpasprj"));
}

#[test]
fn value_options_do_not_consume_following_known_options() {
    let cwd = create_temp_dir("value-option-boundary");
    let cases = [
        (
            vec!["run", "--std-lib", "--version"],
            "Missing directory after `--std-lib`",
        ),
        (
            vec!["build", "--name", "--executable"],
            "Missing application name after `--name`",
        ),
        (
            vec!["test", "--script", "--filter"],
            "Missing path after `--script`",
        ),
        (
            vec!["test", "--filter", "--report"],
            "Missing pattern after `--filter`",
        ),
        (
            vec!["test", "--report", "--timeout"],
            "Missing format after `--report`",
        ),
        (
            vec!["test", "--timeout", "--jobs"],
            "Missing seconds after `--timeout`",
        ),
        (
            vec!["test", "--jobs", "--strict"],
            "Missing count after `--jobs`",
        ),
    ];

    for (args, expected) in cases {
        let args = args.into_iter().map(str::to_string).collect::<Vec<_>>();
        let error = resolve_cli_config(&args, &cwd).expect_err("missing value must fail");
        assert!(error.contains(expected), "unexpected error: {error}");
        assert!(
            error.contains("is an option and cannot be the value"),
            "unexpected error: {error}"
        );
    }

    fs::remove_dir_all(&cwd).expect("temp directory must be removed");
}

#[test]
fn filter_value_may_start_with_a_hyphen_when_it_is_not_a_known_option() {
    let cwd = create_temp_dir("hyphen-filter-value");
    fs::create_dir(cwd.join("tests")).expect("test input directory must be created");
    let result = resolve_cli_config(
        &[
            String::from("test"),
            String::from("--filter"),
            String::from("-menu"),
            String::from("tests"),
        ],
        &cwd,
    );

    match result {
        Ok(ResolvedCli::Test(config)) => assert_eq!(config.filter.as_deref(), Some("-menu")),
        other => panic!("expected test config, got {other:?}"),
    }

    fs::remove_dir_all(&cwd).expect("temp directory must be removed");
}

#[test]
fn resolve_cli_config_parses_test_timeout_flag() {
    let cwd = test_project("test-timeout-flag");
    let result = resolve_cli_config(
        &[
            String::from("test"),
            String::from("--timeout"),
            String::from("30"),
            String::from("demo.fpasprj"),
        ],
        &cwd,
    );

    match result {
        Ok(ResolvedCli::Test(config)) => {
            assert_eq!(config.timeout, Some(std::time::Duration::from_secs(30)));
        }
        other => panic!("expected test config, got {other:?}"),
    }
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");
}

#[test]
fn resolve_cli_config_parses_test_report_json_flag() {
    let cwd = test_project("test-report-json");
    let result = resolve_cli_config(
        &[
            String::from("test"),
            String::from("--report"),
            String::from("json"),
            String::from("demo.fpasprj"),
        ],
        &cwd,
    );

    match result {
        Ok(ResolvedCli::Test(config)) => {
            assert_eq!(config.report, Some(crate::TestReportFormat::Json));
        }
        other => panic!("expected test config, got {other:?}"),
    }
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");
}

#[test]
fn resolve_cli_config_parses_test_jobs_flag() {
    let cwd = test_project("test-jobs-flag");
    let result = resolve_cli_config(
        &[
            String::from("test"),
            String::from("--jobs"),
            String::from("4"),
            String::from("demo.fpasprj"),
        ],
        &cwd,
    );

    match result {
        Ok(ResolvedCli::Test(config)) => assert_eq!(config.jobs, 4),
        other => panic!("expected test config, got {other:?}"),
    }
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");
}

#[test]
fn resolve_cli_config_parses_test_strict_flag() {
    let cwd = test_project("test-strict-flag");
    let result = resolve_cli_config(
        &[
            String::from("test"),
            String::from("--strict"),
            String::from("demo.fpasprj"),
        ],
        &cwd,
    );

    match result {
        Ok(ResolvedCli::Test(config)) => assert!(config.strict),
        other => panic!("expected test config, got {other:?}"),
    }
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");
}

fn test_project(prefix: &str) -> std::path::PathBuf {
    let cwd = create_temp_dir(prefix);
    write_text(&cwd.join("demo.fpasprj"), TEST_PROJECT);
    cwd
}
