//! Integration tests for `fpas test` with test projects.

use crate::cli_test::test_cli;
use crate::test_support::{create_temp_dir, write_text};
use crate::{CliInput, TestCliConfig};

#[test]
fn test_cli_runs_tests_from_test_project_file() {
    let cwd = create_temp_dir("fpas-test-project");
    write_text(
        &cwd.join("tests.fpasprj"),
        "[project]\nname = \"tests\"\nkind = \"test\"\n\n[sources]\ninclude = [\"*.fpas\"]\n",
    );
    write_text(
        &cwd.join("alpha_test.fpas"),
        "program A;\nuses Std.Test;\nbegin AssertTrue(true) end.",
    );
    write_text(
        &cwd.join("beta_test.fpas"),
        "program B;\nuses Std.Test;\nbegin AssertEquals(2, 1 + 1) end.",
    );

    let mut stderr = Vec::new();
    let mut stdout = Vec::new();
    let exit = test_cli(
        TestCliConfig {
            input: CliInput::ProjectFile(cwd.join("tests.fpasprj")),
            cwd: cwd.clone(),
            fail_fast: false,
            list_only: false,
            script_path: None,
            filter: None,
            report: None,
            timeout: None,
            jobs: 1,
        },
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, 0, "stderr={}", String::from_utf8_lossy(&stderr));
    let text = String::from_utf8(stderr).expect("utf-8");
    assert!(text.contains("PASS  alpha_test.fpas"));
    assert!(text.contains("PASS  beta_test.fpas"));
}

#[test]
fn test_cli_runs_tests_from_workspace_test_member() {
    let cwd = create_temp_dir("fpas-test-workspace");
    write_text(
        &cwd.join("root.fpasworkspace"),
        "[workspace]\nname = \"demo\"\nmembers = [\"tests/tests.fpasprj\"]\n",
    );
    write_text(
        &cwd.join("tests/tests.fpasprj"),
        "[project]\nname = \"tests\"\nkind = \"test\"\n\n[sources]\ninclude = [\"*.fpas\"]\n",
    );
    write_text(
        &cwd.join("tests/only_test.fpas"),
        "program O;\nuses Std.Test;\nbegin AssertTrue(true) end.",
    );

    let mut stderr = Vec::new();
    let mut stdout = Vec::new();
    let exit = test_cli(
        TestCliConfig {
            input: CliInput::WorkspaceFile(cwd.join("root.fpasworkspace")),
            cwd: cwd.clone(),
            fail_fast: false,
            list_only: false,
            script_path: None,
            filter: None,
            report: None,
            timeout: None,
            jobs: 1,
        },
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, 0, "stderr={}", String::from_utf8_lossy(&stderr));
    let text = String::from_utf8(stderr).expect("utf-8");
    assert!(text.contains("PASS  only_test.fpas"));
}

#[test]
fn test_cli_uses_manifest_script_override() {
    let cwd = create_temp_dir("fpas-test-manifest-script");
    write_text(
        &cwd.join("tests.fpasprj"),
        "[project]\nname = \"tests\"\nkind = \"test\"\n\n[sources]\ninclude = [\"*.fpas\"]\n\n[test.overrides.\"prompt_test.fpas\"]\nscript = \"prompt.script.toml\"\n",
    );
    write_text(
        &cwd.join("prompt_test.fpas"),
        "program P;\nuses Std.Console, Std.Test;\nbegin\n  var Name: string := ReadLn();\n  AssertTrue(Name = 'Alice')\nend.",
    );
    write_text(
        &cwd.join("prompt.script.toml"),
        "[[event]]\ntype = \"readln\"\nline = \"Alice\"\n",
    );

    let mut stderr = Vec::new();
    let mut stdout = Vec::new();
    let exit = test_cli(
        TestCliConfig {
            input: CliInput::ProjectFile(cwd.join("tests.fpasprj")),
            cwd: cwd.clone(),
            fail_fast: false,
            list_only: false,
            script_path: None,
            filter: Some("prompt".to_string()),
            report: None,
            timeout: None,
            jobs: 1,
        },
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, 0, "stderr={}", String::from_utf8_lossy(&stderr));
    let text = String::from_utf8(stderr).expect("utf-8");
    assert!(text.contains("PASS  prompt_test.fpas"));
}

#[test]
fn test_cli_runs_setup_and_teardown_hooks() {
    let cwd = create_temp_dir("fpas-test-hooks");
    write_text(
        &cwd.join("tests.fpasprj"),
        "[project]\nname = \"tests\"\nkind = \"test\"\n\n[sources]\ninclude = [\"*.fpas\"]\n",
    );
    write_text(
        &cwd.join("fixture.fpas"),
        "unit Tests.Fixture;\nuses Std.Test;\nprocedure Setup();\nbegin AssertTrue(true) end;\nprocedure Teardown();\nbegin AssertTrue(true) end;",
    );
    write_text(
        &cwd.join("demo_test.fpas"),
        "program D;\nuses Std.Test;\nbegin AssertTrue(true) end.",
    );

    let mut stderr = Vec::new();
    let mut stdout = Vec::new();
    let exit = test_cli(
        TestCliConfig {
            input: CliInput::ProjectFile(cwd.join("tests.fpasprj")),
            cwd: cwd.clone(),
            fail_fast: false,
            list_only: false,
            script_path: None,
            filter: None,
            report: None,
            timeout: None,
            jobs: 1,
        },
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, 0, "stderr={}", String::from_utf8_lossy(&stderr));
    let text = String::from_utf8(stderr).expect("utf-8");
    assert!(text.contains("PASS  demo_test.fpas"));
    assert_eq!(text.matches("PASS  demo_test.fpas").count(), 1);
}

#[test]
fn test_cli_fails_when_teardown_hook_fails() {
    let cwd = create_temp_dir("fpas-test-teardown-fail");
    write_text(
        &cwd.join("tests.fpasprj"),
        "[project]\nname = \"tests\"\nkind = \"test\"\n\n[sources]\ninclude = [\"*.fpas\"]\n",
    );
    write_text(
        &cwd.join("fixture.fpas"),
        "unit Tests.Fixture;\nuses Std.Test;\nprocedure Teardown();\nbegin AssertTrue(false) end;",
    );
    write_text(
        &cwd.join("demo_test.fpas"),
        "program D;\nuses Std.Test;\nbegin AssertTrue(true) end.",
    );

    let mut stderr = Vec::new();
    let mut stdout = Vec::new();
    let exit = test_cli(
        TestCliConfig {
            input: CliInput::ProjectFile(cwd.join("tests.fpasprj")),
            cwd: cwd.clone(),
            fail_fast: false,
            list_only: false,
            script_path: None,
            filter: None,
            report: None,
            timeout: None,
            jobs: 1,
        },
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, 1, "stderr={}", String::from_utf8_lossy(&stderr));
    let text = String::from_utf8(stderr).expect("utf-8");
    assert!(text.contains("Teardown hook failed"));
    assert!(
        !text.contains("PASS  demo_test.fpas"),
        "PASS must be deferred until teardown succeeded: {text}"
    );
}

#[test]
fn test_cli_timeout_aborts_hanging_setup_hook() {
    let cwd = create_temp_dir("fpas-test-hook-timeout");
    write_text(
        &cwd.join("tests.fpasprj"),
        "[project]\nname = \"tests\"\nkind = \"test\"\n\n[sources]\ninclude = [\"*.fpas\"]\n",
    );
    write_text(
        &cwd.join("fixture.fpas"),
        "unit Tests.Fixture;\nprocedure Setup();\nbegin\n  while 1 = 1 do\n  begin\n  end\nend;",
    );
    write_text(
        &cwd.join("demo_test.fpas"),
        "program D;\nuses Std.Test;\nbegin AssertTrue(true) end.",
    );

    let mut stderr = Vec::new();
    let mut stdout = Vec::new();
    let exit = test_cli(
        TestCliConfig {
            input: CliInput::ProjectFile(cwd.join("tests.fpasprj")),
            cwd: cwd.clone(),
            fail_fast: false,
            list_only: false,
            script_path: None,
            filter: None,
            report: None,
            timeout: Some(std::time::Duration::from_secs(1)),
            jobs: 1,
        },
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, 3, "stderr={}", String::from_utf8_lossy(&stderr));
    let text = String::from_utf8(stderr).expect("utf-8");
    assert!(text.contains("Setup hook failed"));
    assert!(
        !text.contains("PASS  demo_test.fpas"),
        "test body must not run after the setup hook timed out: {text}"
    );
}
