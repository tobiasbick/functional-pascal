use crate::cli_input::TestCliConfig;
use crate::cli_test::test_cli;
use crate::test_support::{create_temp_dir, write_text};

#[test]
fn test_cli_compares_golden_pixels_for_headless_graph() {
    let cwd = create_temp_dir("fpas-test-expect-pixels");
    write_text(
        &cwd.join("graph_test.fpas"),
        "program G;\nuses Std.Console, Std.Graph, Std.Test;\n\
             mutable var QuitSeen: boolean := false;\n\
             procedure OnPaint(App: Application);\n\
             begin\n\
               Application.Clear(App, $00020408);\n\
               Application.DrawText(App, 2, 2, 'FPAS', $00FFFFFF);\n\
               Application.Present(App)\n\
             end;\n\
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
               var App: Application := Application.OpenForTest(32, 24);\n\
               var EscapeKey: Std.Console.KeyEvent := record\n\
                 kind := Std.Console.KeyKind.Escape;\n\
                 ch := #27;\n\
                 shift := false;\n\
                 ctrl := false;\n\
                 alt := false;\n\
                 meta := false;\n\
               end;\n\
               var Handlers: ApplicationHandlers := record\n\
                 OnPaint := OnPaint;\n\
                 OnKeyPressed := Some(OnKeyPressed);\n\
               end;\n\
               Application.Configure(App, Handlers);\n\
               Application.TestSendKey(App, EscapeKey);\n\
               Application.Run(App);\n\
               AssertTrue(QuitSeen)\n\
             end.",
    );
    write_text(
        &cwd.join("graph_test.expect.pixels"),
        "# size 32 24\n0 0 0x00020408\n2 2 0x00FFFFFF\n",
    );

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = test_cli(
        TestCliConfig {
            input: crate::CliInput::SourceFile(cwd.join("graph_test.fpas")),
            cwd: cwd.clone(),
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
    assert!(text.contains("PASS  graph_test.fpas"));
}

#[test]
fn test_cli_compares_golden_stdout_sidecar() {
    let cwd = create_temp_dir("fpas-test-expect-stdout");
    write_text(
        &cwd.join("echo_test.fpas"),
        "program E;\nuses Std.Console, Std.Test;\nbegin WriteLn('Hello'); WriteLn('World') end.",
    );
    write_text(&cwd.join("echo_test.expect.stdout"), "Hello\nWorld\n");

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = test_cli(
        TestCliConfig {
            input: crate::CliInput::SourceFile(cwd.join("echo_test.fpas")),
            cwd: cwd.clone(),
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
    assert!(text.contains("PASS  echo_test.fpas"));
}

#[test]
fn test_cli_fails_on_stdout_mismatch() {
    let cwd = create_temp_dir("fpas-test-expect-stdout-fail");
    write_text(
        &cwd.join("echo_test.fpas"),
        "program E;\nuses Std.Console, Std.Test;\nbegin WriteLn('Hi') end.",
    );
    write_text(&cwd.join("echo_test.expect.stdout"), "Hello\n");

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = test_cli(
        TestCliConfig {
            input: crate::CliInput::SourceFile(cwd.join("echo_test.fpas")),
            cwd: cwd.clone(),
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

    assert_eq!(exit, 1);
    let text = String::from_utf8(stderr).expect("utf-8");
    assert!(text.contains("stdout mismatch"));
}
