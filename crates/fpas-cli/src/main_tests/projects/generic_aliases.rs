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
  end record;\nend unit;",
    );
    write_text(
        &cwd.join("src/facade.fpas"),
        "unit Repro.Facade;\nuses Repro.Model as ReproModel, Std.Tui as Tui;\npublic type\n  Model = ReproModel.Model;\npublic function NewModel(): Model;\nbegin\n  return Model( Value := 0 );\nend function;\npublic function Update(State: Model; Msg: Tui.TuiMsg; Cmd: Tui.TuiCmdOutput): Model;\nbegin\n  return State;\nend function;\npublic function View(State: Model): Tui.TuiElement;\nbegin\n  return Tui.TuiElementBuilders.MakeLabel('value');\nend function;\nend unit;",
    );
    write_text(
        &cwd.join("src/main.fpas"),
        "program GenericAliasRepro;\nuses Repro.Facade as Facade, Std.Tui as Tui, Std.Test as Test;\nbegin\n  const App: Tui.TuiApplication := Tui.TuiApplication.OpenForTest(Tui.TuiSize.Create(20, 4));\n  var State: Facade.Model := Facade.NewModel();\n  State := App.RunIterations(State, Facade.Update, Facade.View, 0, 0);\n  Test.AssertEquals(0, State.Value);\n  App.Close();\nend.",
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
