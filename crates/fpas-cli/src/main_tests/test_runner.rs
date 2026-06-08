//! Integration tests for `fpas test`.

use crate::cli_test::test_cli;
use crate::test_support::{create_temp_dir, write_text};
use crate::{CliInput, TestCliConfig};

#[test]
fn test_cli_runs_passing_tests_in_directory() {
    let cwd = create_temp_dir("fpas-test-pass");
    write_text(
        &cwd.join("math_test.fpas"),
        "program M;\nuses Std.Test;\nbegin AssertEquals(6, 2 * 3) end.",
    );

    let mut stderr = Vec::new();
    let exit = test_cli(
        TestCliConfig {
            input: CliInput::SourceFile(cwd.clone()),
            cwd,
            fail_fast: false,
            list_only: false,
            script_path: None,
            filter: None,
        },
        &mut stderr,
    );

    assert_eq!(exit, 0);
    let text = String::from_utf8(stderr).expect("utf-8");
    assert!(text.contains("PASS  math_test.fpas"));
}

#[test]
fn test_cli_runs_tui_test_with_sidecar_script() {
    let cwd = create_temp_dir("fpas-tui-test");
    write_text(
        &cwd.join("escape_test.fpas"),
        "program E;\nuses Std.Console, Std.Tui, Std.Test;\n\
         mutable var QuitSeen: boolean := false;\n\
         procedure OnPaint(App: Application); begin end;\n\
         function OnKeyPressed(App: Application; Key: KeyEvent): boolean;\n\
         begin\n\
           if Key.kind = KeyKind.Escape then\n\
           begin\n\
             QuitSeen := true;\n\
             Application.HostRequestQuit(App);\n\
             return true\n\
           end;\n\
           return false\n\
         end;\n\
         begin\n\
           var App: Application := Application.Open();\n\
           var Handlers: ApplicationHandlers := record\n\
             OnPaint := OnPaint;\n\
             OnKeyPressed := Some(OnKeyPressed);\n\
           end;\n\
           Application.Configure(App, Handlers);\n\
           Application.Run(App);\n\
           AssertTrue(QuitSeen)\n\
         end.",
    );
    write_text(
        &cwd.join("escape_test.script.toml"),
        "[[event]]\ntype = \"console_key\"\nkind = \"Escape\"\n",
    );

    let mut stderr = Vec::new();
    let exit = test_cli(
        TestCliConfig {
            input: CliInput::SourceFile(cwd.clone()),
            cwd,
            fail_fast: false,
            list_only: false,
            script_path: None,
            filter: None,
        },
        &mut stderr,
    );

    assert_eq!(exit, 0, "stderr={}", String::from_utf8_lossy(&stderr));
    let text = String::from_utf8(stderr).expect("utf-8");
    assert!(text.contains("PASS  escape_test.fpas"));
}
