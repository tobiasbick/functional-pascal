//! Generic data survives independent unit compilation and concrete alias reexports.
//!
//! **Documentation:** `docs/pascal/language/types/generics.md`.

use super::*;

#[test]
fn generic_data_and_concrete_aliases_link_and_reuse_sidecars() {
    let cwd = create_temp_dir("generic-data-unit-aliases");
    let project = cwd.join("app.fpasprj");
    support::write_program_project_file(&project, "main.fpas", &["*.fpas"]);
    write_text(&cwd.join("data.fpas"), "unit Demo.Data;
        public type Box of (T) = record public Value: T; public Count: integer := 3; end record;
        public type Choice of (T) = enum Present(Value: T); Missing; end enum;
        public function Make of (U)(Value: U): Box of (U); begin return Box(Value := Value); end function;
        public function Wrap of (U)(Value: U): Choice of (U); begin return Choice.Present(Value); end function;
        public function Get of (U)(Item: Box of (U)): U; begin return Item.Value; end function;
        end unit;");
    write_text(
        &cwd.join("facade.fpas"),
        "unit Demo.Facade; uses Demo.Data as Data;
        public type IntegerBox = Data.Box of (integer);
        public type TextBox = Data.Box of (string);
        public type IntegerChoice = Data.Choice of (integer);
        end unit;",
    );
    write_text(
        &cwd.join("main.fpas"),
        r#"program Main; uses Demo.Data as Data; uses Demo.Facade as Facade;
        uses Std.Console as Console;
        const Number: Facade.IntegerBox := Facade.IntegerBox(Value := 42);
        const Text: Facade.TextBox := Data.Make('value');
        const Choice: Facade.IntegerChoice := Facade.IntegerChoice.Present(42);
        const MissingChoice: Facade.IntegerChoice := Facade.IntegerChoice.Missing;
        begin
            Console.WriteLn(Data.Get(Number)); Console.WriteLn(Number.Count);
            Console.WriteLn(Data.Get(Text)); Console.WriteLn(Text.Count);
            Console.WriteLn(Choice = Data.Wrap(42)); Console.WriteLn(MissingChoice <> Choice);
        end program;"#,
    );
    for _ in 0..2 {
        let (exit, stdout, stderr) = support::run_cli_and_capture_output(&project, &cwd);
        assert_eq!(
            (exit, stdout.as_str(), stderr.as_str()),
            (0, "42\n3\nvalue\n3\ntrue\ntrue\n", "")
        );
    }
    fs::remove_dir_all(&cwd).expect("remove temporary project");
}

#[test]
fn nested_decisions_link_imported_generic_payloads_static_constants_and_reused_sidecars() {
    let cwd = create_temp_dir("nested-generic-decisions");
    let project = cwd.join("app.fpasprj");
    support::write_program_project_file(&project, "main.fpas", &["*.fpas"]);
    write_text(
        &cwd.join("model.fpas"),
        "unit Demo.Model;\n        public type Choice of (T) = enum Present(Value: T); Missing; end enum;\n        public const Enabled: boolean := not false;\n        public function Make(): Choice of (Option of (integer)); begin return Choice.Present(Option.Some(42)); end function;\n        end unit;",
    );
    write_text(
        &cwd.join("main.fpas"),
        r#"program Main; uses Demo.Model as Model; uses Std.Console as Console;
        begin
          const Value: integer := case Model.Make() of
            when Model.Choice.Present(Option.Some(const Number)): if Number > 0 then Number else 0 end if;
            when Model.Choice.Present(Option.None): 0;
            when Model.Choice.Missing: 0;
          end case;
          const Flag: Model.Choice of (boolean) := Model.Choice.Present(true);
          Console.WriteLn(Value);
          Console.WriteLn(case Flag of when Model.Choice.Present(Model.Enabled): 42; when Model.Choice.Present(false): 0; when Model.Choice.Missing: 0; end case);
        end program;"#,
    );
    for _ in 0..2 {
        let (exit, stdout, stderr) = support::run_cli_and_capture_output(&project, &cwd);
        assert_eq!(
            (exit, stdout.as_str(), stderr.as_str()),
            (0, "42\n42\n", "")
        );
    }
    write_text(
        &cwd.join("main.fpas"),
        r#"program Main;
uses Demo.Model as Model;
begin
const Value: integer := case Model.Make() of
when Model.Choice.Present(Option.Some(const Number)): Number;
when Model.Choice.Missing: 0;
end case;
end program;"#,
    );
    let args = ["check", "--diagnostics", "json", &project.to_string_lossy()].map(str::to_owned);
    let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(&args, &cwd);
    assert_eq!(exit, 1, "{stderr}");
    assert!(stdout.is_empty());
    let diagnostics = stderr
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("JSON diagnostic"))
        .collect::<Vec<_>>();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic["code"] == "F2011"
                && diagnostic["location"]["start"]["line"] == 4),
        "{stderr}"
    );
    assert!(!stderr.contains("F9001"), "{stderr}");
    fs::remove_dir_all(&cwd).expect("remove temporary project");
}
