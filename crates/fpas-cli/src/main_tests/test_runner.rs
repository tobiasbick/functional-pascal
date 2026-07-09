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
    let mut stdout = Vec::new();
    let exit = test_cli(
        TestCliConfig {
            input: CliInput::SourceFile(cwd.clone()),
            cwd,
            fail_fast: false,
            list_only: false,
            script_path: None,
            filter: None,
            report: None,
            timeout: None,
            jobs: 1,
            strict: false,
        },
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, 0);
    let text = String::from_utf8(stderr).expect("utf-8");
    assert!(text.contains("PASS  math_test.fpas"));
}

#[test]
fn test_cli_runs_native_tui_headless_test() {
    let cwd = create_temp_dir("fpas-tui-test");
    write_text(
        &cwd.join("command_test.fpas"),
        "program E;\nuses Std.Tui, Std.Test;\n\
         mutable var QuitSeen: boolean := false;\n\
         function Bounds(X: integer; Y: integer; Width: integer; Height: integer): Rect;\n\
         begin return record x := X; y := Y; width := Width; height := Height; end end;\n\
         procedure OnCommand(App: Application; CommandId: integer);\n\
         begin if CommandId = CM_OK then QuitSeen := true; Application.Quit(App) end;\n\
         begin\n\
           var App: Application := Application.OpenForTest(80, 25);\n\
           var Win: Std.Tui.Window := Window.New(Bounds(2, 1, 24, 8), 'Test');\n\
           var OkButton: Button := Button.New(Bounds(4, 4, 10, 2), 'OK', CM_OK, true);\n\
           Window.Add(Win, OkButton);\n\
           Desktop.Add(App, Win);\n\
           Application.TestClickButton(App, OkButton);\n\
           Application.Run(App, OnCommand);\n\
           AssertTrue(QuitSeen)\n\
         end.",
    );

    let mut stderr = Vec::new();
    let mut stdout = Vec::new();
    let exit = test_cli(
        TestCliConfig {
            input: CliInput::SourceFile(cwd.clone()),
            cwd,
            fail_fast: false,
            list_only: false,
            script_path: None,
            filter: None,
            report: None,
            timeout: None,
            jobs: 1,
            strict: false,
        },
        &mut stdout,
        &mut stderr,
    );

    assert_eq!(exit, 0, "stderr={}", String::from_utf8_lossy(&stderr));
    let text = String::from_utf8(stderr).expect("utf-8");
    assert!(text.contains("PASS  command_test.fpas"));
}
