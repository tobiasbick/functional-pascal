//! Integration tests for `fpas test` with test projects.

mod exact_selection;

use crate::cli_test::test_cli_with_stderr;
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
        r#"program A;
uses Std.Test as Test;
begin Test.AssertTrue(true); end program;"#,
    );
    write_text(
        &cwd.join("beta_test.fpas"),
        r#"program B;
uses Std.Test as Test;
begin Test.AssertEquals(2, 1 + 1); end program;"#,
    );

    let mut stderr = Vec::new();
    let mut stdout = Vec::new();
    let exit = test_cli_with_stderr(
        TestCliConfig {
            input: CliInput::ProjectFile(cwd.join("tests.fpasprj")),
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
        r#"program O;
uses Std.Test as Test;
begin Test.AssertTrue(true); end program;"#,
    );

    let mut stderr = Vec::new();
    let mut stdout = Vec::new();
    let exit = test_cli_with_stderr(
        TestCliConfig {
            input: CliInput::WorkspaceFile(cwd.join("root.fpasworkspace")),
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
        r#"program P;
uses Std.Console as Console; uses Std.Test as Test;
begin
  const Name: string := Console.ReadLn();
  Test.AssertTrue(Name = 'Alice');
end program;"#,
    );
    write_text(
        &cwd.join("prompt.script.toml"),
        "[[event]]\ntype = \"readln\"\nline = \"Alice\"\n",
    );

    let mut stderr = Vec::new();
    let mut stdout = Vec::new();
    let exit = test_cli_with_stderr(
        TestCliConfig {
            input: CliInput::ProjectFile(cwd.join("tests.fpasprj")),
            cwd: cwd.clone(),
            fail_fast: false,
            list_only: false,
            script_path: None,
            filter: Some("prompt".to_string()),
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
        r#"unit Tests.Fixture;
uses Std.Test as Test;
public procedure Setup();
begin Test.AssertTrue(true); end procedure;
public procedure Teardown();
begin Test.AssertTrue(true); end procedure;
end unit;
"#,
    );
    write_text(
        &cwd.join("demo_test.fpas"),
        r#"program D;
uses Std.Test as Test;
begin Test.AssertTrue(true); end program;"#,
    );

    let mut stderr = Vec::new();
    let mut stdout = Vec::new();
    let exit = test_cli_with_stderr(
        TestCliConfig {
            input: CliInput::ProjectFile(cwd.join("tests.fpasprj")),
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
        r#"unit Tests.Fixture;
uses Std.Test as Test;
public procedure Teardown();
begin Test.AssertTrue(false); end procedure;
end unit;
"#,
    );
    write_text(
        &cwd.join("demo_test.fpas"),
        r#"program D;
uses Std.Test as Test;
begin Test.AssertTrue(true); end program;"#,
    );

    let mut stderr = Vec::new();
    let mut stdout = Vec::new();
    let exit = test_cli_with_stderr(
        TestCliConfig {
            input: CliInput::ProjectFile(cwd.join("tests.fpasprj")),
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
        r#"unit Tests.Fixture;
public procedure Setup();
begin
  while 1 = 1 do
  begin null;
  end; end while;
end procedure;
end unit;
"#,
    );
    write_text(
        &cwd.join("demo_test.fpas"),
        r#"program D;
uses Std.Test as Test;
begin Test.AssertTrue(true); end program;"#,
    );

    let mut stderr = Vec::new();
    let mut stdout = Vec::new();
    let exit = test_cli_with_stderr(
        TestCliConfig {
            input: CliInput::ProjectFile(cwd.join("tests.fpasprj")),
            cwd: cwd.clone(),
            fail_fast: false,
            list_only: false,
            script_path: None,
            filter: None,
            files: Vec::new(),
            report: None,
            timeout: Some(std::time::Duration::from_secs(1)),
            jobs: 1,
            strict: false,
            show_output: false,
            diagnostics: Default::default(),
            standard_library: None,
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

#[test]
fn test_cli_reports_runtime_errors_of_units_linked_out_of_graph_order() {
    for jobs in [1, 2] {
        let cwd = create_temp_dir("fpas-test-unit-link-order");
        write_text(
            &cwd.join("tests.fpasprj"),
            "[project]\nname = \"tests\"\nkind = \"test\"\n\n[sources]\ninclude = [\"*.fpas\"]\n",
        );
        // `util.fpas` precedes `zeta.fpas` in the unit graph, but the linker emits App.Zeta first.
        write_text(
            &cwd.join("util.fpas"),
            r#"unit App.Util;
uses App.Zeta as Zeta;
public procedure Trigger();
begin
  if Zeta.Seven() = 7 then panic('util failure'); end if;
end procedure;
end unit;

"#,
        );
        write_text(
            &cwd.join("zeta.fpas"),
            r#"unit App.Zeta;
public function Seven(): integer;
begin
  return 7;
end function;
end unit;

"#,
        );
        write_text(
            &cwd.join("trigger_test.fpas"),
            r#"program TriggerTest;
uses App.Util as Util;
begin
  Util.Trigger();
end program;"#,
        );

        let mut stderr = Vec::new();
        let mut stdout = Vec::new();
        let exit = test_cli_with_stderr(
            TestCliConfig {
                input: CliInput::ProjectFile(cwd.join("tests.fpasprj")),
                cwd: cwd.clone(),
                fail_fast: false,
                list_only: false,
                script_path: None,
                filter: None,
                files: Vec::new(),
                report: None,
                timeout: None,
                jobs,
                strict: false,
                show_output: false,
                diagnostics: Default::default(),
                standard_library: None,
            },
            &mut stdout,
            &mut stderr,
        );

        let text = String::from_utf8(stderr).expect("utf-8");
        assert_ne!(exit, 0, "jobs={jobs}");
        assert!(text.contains("util.fpas:5:"), "jobs={jobs}: {text}");
        assert!(text.contains("util failure"), "jobs={jobs}: {text}");
        assert!(!text.contains("zeta.fpas:"), "jobs={jobs}: {text}");
    }
}
