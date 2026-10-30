//! Public cached-view behavior and rejected lifecycle/validation misuse.
use super::*;

#[test]
fn interactive_demo_preserves_focus_then_confirms_and_quits() {
    let source = include_str!("../../../../../examples/pascal/tui/interactive_demo.fpas");
    let (declarations, _) = source.rsplit_once("\nbegin\n").expect("demo main block");
    let declarations = declarations.replace(
        "uses Std.Ui.Terminal as Tui;",
        "uses Std.Ui.Terminal as Tui, Std.Test as Test;",
    );
    let cwd = create_temp_dir("tui-demo-subviews");
    let entry = cwd.join("demo.fpas");
    write_text(
        &entry,
        &format!(
            r#"{declarations}
begin
  const App := Tui.TuiApplication.OpenForTest(Tui.TuiSize.Create(80, 24));
  var State := DemoModel(Text := 'hello', Caret := 5, Confirming := false, Focus := 1);
  App.InjectKeyForTest(Tui.TuiKeyEvent(kind := Tui.TuiKeyKind.Tab, ch := '', shift := false, ctrl := false, alt := false, meta := false));
  State := App.RunIterations(State, UpdateDemo, ViewDemo, 1, 0);
  Test.AssertEquals(2, State.Focus);
  App.InjectKeyForTest(Tui.TuiKeyEvent(kind := Tui.TuiKeyKind.Enter, ch := '', shift := false, ctrl := false, alt := false, meta := false));
  State := App.RunIterations(State, UpdateDemo, ViewDemo, 1, 0);
  Test.AssertTrue(State.Confirming);
  Test.AssertEquals(3, State.Focus);
  App.InjectKeyForTest(Tui.TuiKeyEvent(kind := Tui.TuiKeyKind.Enter, ch := '', shift := false, ctrl := false, alt := false, meta := false));
  State := App.RunIterations(State, UpdateDemo, ViewDemo, 1, 0);
  App.Close();
end.
"#
        ),
    );
    let (exit, stdout, stderr) =
        support::run_program_with_graph_and_capture_output(&entry, repo_tui_program_graph());
    fs::remove_dir_all(cwd).expect("demo fixture must be removed");
    assert_eq!(exit, 0, "{stdout}\n{stderr}");
}

#[test]
fn cached_subviews_preserve_dependencies_lifetimes_output_and_routing() {
    for name in ["cache", "dependencies", "render", "routing", "lifetime"] {
        let (exit, stdout, stderr) =
            run_repo_std_program(&format!("tests/stdlib/tui/subview_{name}_test.fpas"));
        assert_eq!(exit, 0, "{name}: {stdout}\n{stderr}");
    }
}

#[test]
fn cached_subviews_reject_outside_render_recursive_builds_and_duplicate_controls() {
    const DECLARATIONS: &str = r#"
uses Std.Ui.Terminal as Tui;
var Alias: option of Tui.TuiSubview of integer := None;
function Build(Mode: integer): Tui.TuiElement;
begin
  if Mode = 1 then return Alias.Unwrap().Render(Mode); end if;
  return Tui.TuiElementBuilders.MakeButton(Tui.TuiControlId.Create(1), 'Go', Tui.TuiAction.Create(1));
end function;
function Equal(Previous: integer; Current: integer): boolean;
begin return Previous = Current; end function;
const Pane: Tui.TuiSubview of integer := Tui.TuiSubview.Create(Build, Equal);
function Update(State: integer; Msg: Tui.TuiMsg; Cmd: Tui.TuiCmdOutput): integer;
begin return State; end function;
function View(State: integer): Tui.TuiElement;
begin
  const Tree := Pane.Render(State);
  if State = 2 then return Tui.TuiElementBuilders.MakeColumn([Tree, Tree]); end if;
  return Tree;
end function;
"#;
    for (body, expected) in [
        (
            "discard Pane.Render(0);",
            "Tui subviews can only render inside View",
        ),
        (
            "Alias := Some(Pane); const App := Tui.TuiApplication.OpenForTest(Tui.TuiSize.Create(10, 2)); discard App.RunIterations(1, Update, View, 0, 0);",
            "Tui subview callbacks must not reenter the same subview",
        ),
        (
            "const App := Tui.TuiApplication.OpenForTest(Tui.TuiSize.Create(10, 2)); discard App.RunIterations(2, Update, View, 0, 0);",
            "Tui control id must be unique in one tree: 1",
        ),
    ] {
        let cwd = create_temp_dir("tui-subview-rejection");
        let entry = cwd.join("main.fpas");
        write_text(
            &entry,
            &format!("program RejectedSubview; {DECLARATIONS} begin {body} end."),
        );
        let (exit, _stdout, stderr) =
            support::run_program_with_graph_and_capture_output(&entry, repo_tui_program_graph());
        fs::remove_dir_all(cwd).expect("subview fixture must be removed");
        assert_ne!(exit, 0);
        assert!(stderr.contains(expected), "{stderr}");
    }
}
