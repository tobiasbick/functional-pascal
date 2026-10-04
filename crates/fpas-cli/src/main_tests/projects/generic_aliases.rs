//! Generic inference through a public alias facade and imported callbacks.
//!
//! **Documentation:** `docs/pascal/language/types/type-aliases.md`.

use super::*;

#[test]
fn generic_tui_callbacks_accept_a_public_model_alias() {
    let cwd = create_temp_dir("generic-tui-model-alias");
    let project = cwd.join("repro.fpasprj");
    support::write_program_project_file(&project, "src/main.fpas", &["src/**/*.fpas"]);
    write_text(
        &cwd.join("src/model.fpas"),
        r#"unit Repro.Model;

  public type Model = record
    public Value: integer;
  end record;
end unit;
"#,
    );
    write_text(
        &cwd.join("src/facade.fpas"),
        r#"unit Repro.Facade;
uses Repro.Model as Model2; uses Std.Tui as Tui;
uses Std.Tui.Elements as Elements;

  public type Model = Model2.Model;
public function NewModel(): Model;
begin
  return Model(Value := 0);
end function;
public function Update(State: Model; Msg: Tui.TuiMsg; Cmd: Tui.TuiCmdOutput): Model;
begin
  return State;
end function;
public function View(State: Model): Tui.TuiElement;
begin
  return Elements.TuiElementMakeLabel('value');
end function;
end unit;
"#,
    );
    write_text(
        &cwd.join("src/main.fpas"),
        r#"program GenericAliasRepro;
uses Repro.Facade as Facade; uses Std.Tui as Tui; uses Std.Test as Test;
uses Std.Tui.Geometry as Geometry;
uses Std.Tui.Runtime as Runtime;
begin
  const App: Tui.TuiApplication := Runtime.TuiApplicationOpenForTest(Geometry.TuiSizeCreate(20, 4));
   var State: Facade.Model := Facade.NewModel();
  State := Runtime.TuiApplicationRunIterations(App, State, Facade.Update, Facade.View, 0, 0);
  Test.AssertEquals(0, State.Value);
  Runtime.TuiApplicationClose(App);
end program;"#,
    );
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("repository root");
    for command in ["check", "run"] {
        let (code, _, stderr) = support::run_cli_args_and_capture_output(
            &[
                command.to_string(),
                "--std-lib".to_string(),
                root.join("lib").to_string_lossy().into_owned(),
                project.to_string_lossy().into_owned(),
            ],
            &cwd,
        );
        assert_eq!(code, 0, "{command}: {stderr}");
    }
    fs::remove_dir_all(&cwd).expect("remove fixture");
}
