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
        "unit Repro.Model;
public type
  Model = record
    public Value: integer;
  end;",
    );
    write_text(
        &cwd.join("src/facade.fpas"),
        "unit Repro.Facade;
uses Repro.Model, Std.Tui;
public type
  Model = Repro.Model.Model;
public function NewModel(): Model;
begin
  return record Value := 0; end
end;
public function Update(State: Model; Msg: TuiMsg; Cmd: TuiCmdOutput): Model;
begin
  return State
end;
public function View(State: Model): TuiElement;
begin
  return TuiElementBuilders.MakeLabel('value')
end;",
    );
    write_text(
        &cwd.join("src/main.fpas"),
        "program GenericAliasRepro;
uses Repro.Facade, Std.Tui, Std.Test;
begin
  var App: TuiApplication := TuiApplication.OpenForTest(TuiSize.Create(20, 4));
  mutable var State: Model := NewModel();
  State := App.RunIterations(State, Update, View, 0, 0);
  AssertEquals(0, State.Value);
  App.Close()
end.",
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
