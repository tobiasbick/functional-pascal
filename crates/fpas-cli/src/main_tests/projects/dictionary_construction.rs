//! Nominal key layouts and key rejection survive independently compiled units.

use super::*;

#[test]
fn imported_record_keys_normalize_with_reused_layouts_and_sidecars() {
    let cwd = create_temp_dir("imported-dictionary-keys");
    let project = cwd.join("app.fpasprj");
    support::write_program_project_file(&project, "main.fpas", &["*.fpas"]);
    write_text(&cwd.join("model.fpas"), "unit Demo.Model;
        public type Key of (T: Equatable) = record public Value: T; end record;
        public type IntegerKey = Key of (integer);
        public function Make(Value: integer): IntegerKey; begin return IntegerKey(Value := Value); end function;
        end unit;");
    write_text(
        &cwd.join("main.fpas"),
        r#"program Main;
        uses Demo.Model as Model; uses Std.Console as Console; uses Std.Dictionaries as Dictionaries;
        begin const Values: dict of (Model.IntegerKey, integer) := [Model.Make(1): 1, Model.IntegerKey(Value := 1): 42];
          Console.WriteLn(Dictionaries.Length(Values)); Console.WriteLn(Values[Model.Make(1)]);
        end program;"#,
    );
    for _ in 0..2 {
        let (exit, stdout, stderr) = support::run_cli_and_capture_output(&project, &cwd);
        assert_eq!((exit, stdout.as_str(), stderr.as_str()), (0, "1\n42\n", ""));
    }
    fs::remove_dir_all(&cwd).expect("remove temporary project");
}

#[test]
fn imported_aliases_cannot_hide_unsupported_key_components() {
    let cwd = create_temp_dir("unsupported-imported-dictionary-key");
    let project = cwd.join("app.fpasprj");
    support::write_program_project_file(&project, "main.fpas", &["*.fpas"]);
    write_text(
        &cwd.join("model.fpas"),
        "unit Demo.Model;
        public type Key = record public Action: function(): integer; end record;
        public type Alias = Key; end unit;",
    );
    write_text(
        &cwd.join("main.fpas"),
        r#"program Main;
uses Demo.Model as Model;
const Values: dict of (Model.Alias, integer) := [:];
begin null; end program;
"#,
    );
    let args = ["check", "--diagnostics", "json", &project.to_string_lossy()].map(str::to_owned);
    let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(&args, &cwd);
    assert_eq!(exit, 1, "{stderr}");
    assert!(stdout.is_empty());
    assert!(
        stderr.lines().any(|line| {
            let diagnostic: serde_json::Value =
                serde_json::from_str(line).expect("JSON diagnostic");
            diagnostic["code"] == "F2006"
                && diagnostic["location"]["start"]["line"] == 3
                && diagnostic["message"]
                    .as_str()
                    .is_some_and(|message| message.contains("Unsupported dictionary key type"))
        }),
        "{stderr}"
    );
    fs::remove_dir_all(&cwd).expect("remove temporary project");
}
