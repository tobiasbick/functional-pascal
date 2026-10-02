use crate::cli_input::TestCliConfig;
use crate::cli_test::test_cli_with_stderr;
use crate::test_support::{FailingWriter, create_temp_dir, write_text};

#[test]
fn test_cli_runs_matching_tests_in_directory() {
    let cwd = create_temp_dir("fpas-test-dir");
    write_text(
        &cwd.join("pass_test.fpas"),
        r#"program P;
uses Std.Test as Test;
begin Test.AssertTrue(true); end program;"#,
    );
    write_text(
        &cwd.join("fail_test.fpas"),
        r#"program F;
uses Std.Test as Test;
begin Test.AssertTrue(false); end program;"#,
    );
    write_text(
        &cwd.join("helper.fpas"),
        "unit H;\nprocedure X; begin null; end procedure; end unit;",
    );

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = test_cli_with_stderr(
        TestCliConfig {
            input: crate::CliInput::SourceFile(cwd.clone()),
            cwd: cwd.clone(),
            fail_fast: false,
            list_only: false,
            script_path: None,
            filter: None,
            files: Vec::new(),
            report: None,
            timeout: None,
            jobs: 1,
            strict: false,
            show_output: false,
            diagnostics: Default::default(),
            standard_library: None,
        },
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, 1);
    let text = String::from_utf8(stderr).expect("utf-8");
    assert!(text.contains("PASS  pass_test.fpas"));
    assert!(text.contains("FAIL  fail_test.fpas"));
    assert!(!text.contains("helper.fpas"));
}

#[test]
fn test_cli_list_only_prints_paths_without_running() {
    let cwd = create_temp_dir("fpas-test-list");
    write_text(
        &cwd.join("one_test.fpas"),
        r#"program O;
uses Std.Test as Test;
begin Test.AssertTrue(false); end program;"#,
    );

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = test_cli_with_stderr(
        TestCliConfig {
            input: crate::CliInput::SourceFile(cwd.clone()),
            cwd,
            fail_fast: false,
            list_only: true,
            script_path: None,
            filter: None,
            files: Vec::new(),
            report: None,
            timeout: None,
            jobs: 1,
            strict: false,
            show_output: false,
            diagnostics: Default::default(),
            standard_library: None,
        },
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, 0);
    let listed = String::from_utf8(stdout).expect("utf-8");
    assert!(listed.contains("one_test.fpas"));
    let progress = String::from_utf8(stderr).expect("utf-8");
    assert!(!progress.contains("FAIL"));
    assert!(!progress.contains("one_test.fpas"));
}

#[test]
fn test_cli_list_only_fails_when_stdout_cannot_be_written() {
    let cwd = create_temp_dir("fpas-test-list-write-failure");
    write_text(
        &cwd.join("one_test.fpas"),
        r#"program O;
uses Std.Test as Test;
begin Test.AssertTrue(true); end program;"#,
    );
    for mut stdout in [FailingWriter::immediately(), FailingWriter::after(8)] {
        let mut stderr = Vec::new();
        let exit = test_cli_with_stderr(
            TestCliConfig {
                input: crate::CliInput::SourceFile(cwd.clone()),
                cwd: cwd.clone(),
                fail_fast: false,
                list_only: true,
                script_path: None,
                filter: None,
                files: Vec::new(),
                report: None,
                timeout: None,
                jobs: 1,
                strict: false,
                show_output: false,
                diagnostics: Default::default(),
                standard_library: None,
            },
            &mut stdout,
            &mut stderr,
        );
        let stderr = String::from_utf8(stderr).expect("stderr must be UTF-8");

        assert_eq!(exit, 1);
        assert!(
            stderr.contains("Cannot write test file list to stdout"),
            "unexpected stderr: {stderr}"
        );
    }
    std::fs::remove_dir_all(&cwd).expect("temp directory must be removed");
}

#[test]
fn test_cli_filter_runs_matching_tests_only() {
    let cwd = create_temp_dir("fpas-test-filter");
    write_text(
        &cwd.join("menu_test.fpas"),
        r#"program M;
uses Std.Test as Test;
begin Test.AssertTrue(true); end program;"#,
    );
    write_text(
        &cwd.join("other_test.fpas"),
        r#"program O;
uses Std.Test as Test;
begin Test.AssertTrue(false); end program;"#,
    );

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = test_cli_with_stderr(
        TestCliConfig {
            input: crate::CliInput::SourceFile(cwd.clone()),
            cwd,
            fail_fast: false,
            list_only: false,
            script_path: None,
            filter: Some("menu".to_string()),
            files: Vec::new(),
            report: None,
            timeout: None,
            jobs: 1,
            strict: false,
            show_output: false,
            diagnostics: Default::default(),
            standard_library: None,
        },
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, 0);
    let text = String::from_utf8(stderr).expect("utf-8");
    assert!(text.contains("PASS  menu_test.fpas"));
    assert!(!text.contains("other_test.fpas"));
}
#[test]
fn test_cli_jobs_runs_tests_in_parallel_mode() {
    let cwd = create_temp_dir("fpas-test-jobs");
    write_text(
        &cwd.join("one_test.fpas"),
        r#"program O;
uses Std.Test as Test;
begin Test.AssertTrue(true); end program;"#,
    );
    write_text(
        &cwd.join("two_test.fpas"),
        r#"program T;
uses Std.Test as Test;
begin Test.AssertEquals(2, 1 + 1); end program;"#,
    );

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = test_cli_with_stderr(
        TestCliConfig {
            input: crate::CliInput::SourceFile(cwd.clone()),
            cwd,
            fail_fast: false,
            list_only: false,
            script_path: None,
            filter: None,
            files: Vec::new(),
            report: None,
            timeout: None,
            jobs: 2,
            strict: false,
            show_output: false,
            diagnostics: Default::default(),
            standard_library: None,
        },
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, 0, "stderr={}", String::from_utf8_lossy(&stderr));
    let text = String::from_utf8(stderr).expect("utf-8");
    assert!(text.contains("PASS  one_test.fpas"));
    assert!(text.contains("PASS  two_test.fpas"));
}
#[test]
fn test_cli_fail_fast_records_not_run_tests() {
    let cwd = create_temp_dir("fpas-test-fail-fast");
    write_text(
        &cwd.join("aaa_pass_test.fpas"),
        r#"program P;
uses Std.Test as Test;
begin Test.AssertTrue(true); end program;"#,
    );
    write_text(
        &cwd.join("bbb_fail_test.fpas"),
        r#"program F;
uses Std.Test as Test;
begin Test.AssertTrue(false); end program;"#,
    );
    write_text(
        &cwd.join("ccc_later_test.fpas"),
        r#"program L;
uses Std.Test as Test;
begin Test.AssertTrue(true); end program;"#,
    );

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = test_cli_with_stderr(
        TestCliConfig {
            input: crate::CliInput::SourceFile(cwd.clone()),
            cwd: cwd.clone(),
            fail_fast: true,
            list_only: false,
            script_path: None,
            filter: None,
            files: Vec::new(),
            report: None,
            timeout: None,
            jobs: 1,
            strict: false,
            show_output: false,
            diagnostics: Default::default(),
            standard_library: None,
        },
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, 1);
    let text = String::from_utf8(stderr).expect("utf-8");
    assert!(text.contains("PASS  aaa_pass_test.fpas"));
    assert!(text.contains("FAIL  bbb_fail_test.fpas"));
    assert!(text.contains("not run, --fail-fast"));
    assert!(text.contains("1 not run"));
    assert!(!text.contains("PASS  ccc_later_test.fpas"));
}

#[test]
fn parallel_fail_fast_stops_after_a_link_context_error() {
    let cwd = create_temp_dir("fpas-test-parallel-link-fail-fast");
    let broken_dir = cwd.join("aaa_broken");
    write_text(
        &broken_dir.join("broken.fpasprj"),
        "[project]\nname = \"broken\"\nkind = \"test\"\n",
    );
    write_text(
        &broken_dir.join("broken_test.fpas"),
        r#"program Broken;  uses Std.Test as Test; begin Test.AssertTrue(true); end program;"#,
    );
    write_text(
        &cwd.join("zzz_later_test.fpas"),
        r#"program Later;  uses Std.Test as Test; begin Test.AssertTrue(true); end program;"#,
    );

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = test_cli_with_stderr(
        TestCliConfig {
            input: crate::CliInput::SourceFile(cwd.clone()),
            cwd,
            fail_fast: true,
            list_only: false,
            script_path: None,
            filter: None,
            files: Vec::new(),
            report: None,
            timeout: None,
            jobs: 2,
            strict: false,
            show_output: false,
            diagnostics: Default::default(),
            standard_library: None,
        },
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, 2);
    let text = String::from_utf8(stderr).expect("utf-8");
    assert!(text.contains("FAIL  broken_test.fpas"));
    assert!(text.contains("not run, --fail-fast"));
    assert!(!text.contains("PASS  zzz_later_test.fpas"));
}
