//! Structured scripted input through the debugger and isolated test worker.

use super::*;

#[test]
fn structured_script_quits_the_interactive_tui_through_its_console_pump() {
    let f = Fixture::new(
        r#"program ScriptedTui;
uses Std.Test as Test, Std.Ui.Terminal as Tui;

function Update(State: integer; Msg: Tui.TuiMsg; Cmd: Tui.TuiCmdOutput): integer;
begin
  case Msg of
    when Tui.TuiMsg.Key(const Key):
      begin
        if (Key.kind = Tui.TuiKeyKind.Character) and (Key.ch = 'q') then
          Cmd.Set(Tui.TuiCmd.Quit);
          return State + 1;
        end if;
        return State;
      end;
    when Tui.TuiMsg.Started, Tui.TuiMsg.Tick(_), Tui.TuiMsg.Pointer(_), Tui.TuiMsg.Resize(_), Tui.TuiMsg.FocusChanged(_, _), Tui.TuiMsg.Action(_, _), Tui.TuiMsg.TextChanged(_, _, _, _), Tui.TuiMsg.TextAreaChanged(_, _, _, _, _), Tui.TuiMsg.CheckChanged(_, _, _), Tui.TuiMsg.SelectionChanged(_, _, _), Tui.TuiMsg.ScrollChanged(_, _, _), Tui.TuiMsg.MenuChanged(_, _, _), Tui.TuiMsg.BackgroundFailed(_, _, _, _, _, _), Tui.TuiMsg.QuitRequested:
      return State;
  end case;
end function;

function View(State: integer): Tui.TuiElement;
begin
  return Tui.TuiElementBuilders.MakeLabel('waiting for q');
end function;

begin
  const Final := Tui.TuiApplication.Run(0, Update, View);
  Test.AssertEquals(1, Final);
end."#,
    );
    f.write(
        "input.toml",
        "[[event]]\ntype = 'console_resize'\nwidth = 20\nheight = 5\n\n\
         [[event]]\ntype = 'console_key'\nkind = 'Character'\nch = 'q'\n",
    );
    let r = f.run(
        &[
            "debug",
            "main.fpas",
            "--script",
            "input.toml",
            "--timeout",
            "15",
        ],
        0,
    );
    assert_eq!(r["outcome"], "completed");
}

#[test]
fn test_sidecar_preserves_structured_events_and_consumes_keys_once() {
    let f = Fixture::new(HELLO);
    f.write(
        "events_test.fpas",
        r#"program ScriptedEvents;
uses Std.Console as Console, Std.Test as Test;
begin
  Test.AssertEquals(87, Console.ScreenWidth());
  Test.AssertEquals(31, Console.ScreenHeight());
  const First := Console.ReadEventTimeout(0).Unwrap();
  Test.AssertTrue(First.kind = Console.EventKind.Resize);
  Test.AssertEquals(100, First.width);
  Test.AssertEquals(30, First.height);
  const Character := Console.ReadEventTimeout(0).Unwrap();
  Test.AssertTrue(Character.kind = Console.EventKind.Key);
  Test.AssertTrue(Character.key.kind = Console.KeyKind.Character);
  Test.AssertEquals('é', Character.key.ch);
  Test.AssertTrue(Character.key.shift and Character.key.ctrl and Character.key.alt and Character.key.meta);
  const Escape := Console.ReadKeyEvent();
  Test.AssertTrue(Escape.kind = Console.KeyKind.Escape);
  Test.AssertFalse(Escape.shift or Escape.ctrl or Escape.alt or Escape.meta);
  const Second := Console.ReadEventTimeout(0).Unwrap();
  Test.AssertTrue(Second.kind = Console.EventKind.Resize);
  Test.AssertEquals(87, Second.width);
  Test.AssertEquals(31, Second.height);
  Test.AssertTrue(Console.ReadKeyEvent().kind = Console.KeyKind.F12);
  Test.AssertFalse(Console.KeyPressed());
  Test.AssertFalse(Console.EventPending());
end."#,
    );
    f.write(
        "events_test.script.toml",
        r#"[[event]]
type = "console_resize"
width = 100
height = 30

[[event]]
type = "console_key"
kind = "Character"
ch = "é"
shift = true
ctrl = true
alt = true
meta = true

[[event]]
type = "console_key"
kind = "Escape"

[[event]]
type = "console_resize"
width = 87
height = 31

[[event]]
type = "console_key"
kind = "F12"
"#,
    );
    let output = f
        .command()
        .args([
            "test",
            "events_test.fpas",
            "--report",
            "json",
            "--timeout",
            "5",
        ])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let r: Value = serde_json::from_slice(&output.stdout).expect("test report");
    assert_eq!(r["summary"]["total"], 1);
    assert_eq!(r["summary"]["passed"], 1);
}

#[test]
fn malformed_sidecar_preserves_pre_execution_diagnostics() {
    let f = Fixture::new(HELLO);
    f.write("invalid_test.fpas", "program InvalidInput; begin end.");
    f.write(
        "invalid_test.script.toml",
        "[[event]]\ntype = 'console_key'\nkind = 'Character'\nch = 'ab'\n",
    );
    let output = f
        .command()
        .args([
            "test",
            "invalid_test.fpas",
            "--report",
            "json",
            "--timeout",
            "5",
        ])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(!output.status.success(), "{output:?}");
    let r: Value = serde_json::from_slice(&output.stdout).expect("test report");
    assert_eq!(r["summary"]["compile_errors"], 1, "{output:?}");
    assert_eq!(r["summary"]["runtime_errors"], 0);
    assert_eq!(output.status.code(), Some(2));
    let diagnostics = String::from_utf8(output.stderr).expect("UTF-8 diagnostics");
    assert!(diagnostics.contains("`ch`"), "{diagnostics}");
    assert!(
        diagnostics.contains("invalid_test.script.toml"),
        "{diagnostics}"
    );
    assert!(
        !diagnostics.contains("test worker failed unexpectedly"),
        "{diagnostics}"
    );
}
