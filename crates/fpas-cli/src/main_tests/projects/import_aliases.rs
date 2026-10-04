//! Source alias identities across enum facades, callable values, and tasks.
//!
//! **Documentation:** `docs/pascal/program-structure/units.md`.

use super::*;

#[test]
fn imported_global_names_do_not_replace_alias_roots() {
    let cwd = create_temp_dir("alias-storage-roots");
    let project = cwd.join("app.fpasprj");
    support::write_program_project_file(&project, "main.fpas", &["*.fpas"]);
    write_text(
        &cwd.join("model.fpas"),
        "unit Demo.Data;
        public const Values: array of (integer) := [1];
        public var Math: integer := 7;
        public var Console: integer := 9;
        public type Data = record public Amount: integer; end record;
        public var Item: Data := Data(Amount := 0);
        public var Items: array of (Data) := [Data(Amount := 0)];
        end unit;",
    );
    write_text(
        &cwd.join("main.fpas"),
        "program Main;
        uses Demo.Data as vAlUeS;
        uses Std.Math as Math; uses Std.Console as Console;
        procedure Set(var Value: integer); begin Value := 5; end procedure;
        begin
          const Copy := Values.Values;
          Values.Item.Amount := 4;
          Set(var Values.Items[0].Amount);
          Console.WriteLn(Copy[0]);
          Console.WriteLn(Values.Item.Amount); Console.WriteLn(Values.Items[0].Amount);
          Console.WriteLn(Math.Pi > 3.0); Console.WriteLn(Console.White + Console.Blink = 143);
          Console.WriteLn(Values.Math); Console.WriteLn(Values.Console);
        end program;",
    );
    for _ in 0..2 {
        let (exit, stdout, stderr) = support::run_cli_and_capture_output(&project, &cwd);
        assert_eq!(
            (exit, stdout.as_str(), stderr.as_str()),
            (0, "1\n4\n5\ntrue\ntrue\n7\n9\n", "")
        );
    }
    let args = ["build".to_owned(), project.to_string_lossy().into_owned()];
    let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(&args, &cwd);
    assert_eq!(exit, 0, "{stderr}");
    assert!(stdout.contains("Reused program"), "{stdout}");
    for (statement, code) in [
        ("Values.Values[0] := 2;", "F2005"),
        ("const Copy := Values;", "F2003"),
        ("const Copy := Item;", "F2003"),
        ("const Copy := Demo.Data.Values;", "F2003"),
    ] {
        write_text(
            &cwd.join("main.fpas"),
            &format!("program Main; uses Demo.Data as Values; begin {statement} end program;"),
        );
        let args = ["check".to_owned(), project.to_string_lossy().into_owned()];
        let (exit, _, stderr) = support::run_cli_args_and_capture_output(&args, &cwd);
        assert_eq!(exit, 1, "{statement}: {stderr}");
        assert!(stderr.contains(code), "{statement}: {stderr}");
        assert!(!stderr.contains("F9001"), "{statement}: {stderr}");
    }
    fs::remove_dir_all(&cwd).expect("remove alias storage project");
}

#[test]
fn mutable_import_alias_member_paths_preserve_snapshots_and_var_calls() {
    let cwd = create_temp_dir("mutable-alias-storage");
    let project = cwd.join("app.fpasprj");
    support::write_program_project_file(&project, "main.fpas", &["*.fpas"]);
    write_text(
        &cwd.join("model.fpas"),
        "unit Demo.Data;
        public var Values: array of (integer) := [1];
        end unit;",
    );
    write_text(
        &cwd.join("main.fpas"),
        "program Main;
        uses Demo.Data as Values; uses Std.Arrays as Arrays; uses Std.Console as Console;
        procedure Set(var Value: integer); begin Value := 3; end procedure;
        begin
          const Before := Values.Values;
          Arrays.Push(var Values.Values, 2); Set(var Values.Values[0]);
          Console.WriteLn(Before[0]); Console.WriteLn(Values.Values[0]);
          Console.WriteLn(Values.Values[1]);
        end program;",
    );
    for _ in 0..2 {
        let (exit, stdout, stderr) = support::run_cli_and_capture_output(&project, &cwd);
        assert_eq!(
            (exit, stdout.as_str(), stderr.as_str()),
            (0, "1\n3\n2\n", "")
        );
    }
    assert!(cwd.join("model.fpascu").is_file());
    fs::remove_dir_all(&cwd).expect("remove mutable alias storage project");
}

#[test]
fn aliases_preserve_enum_reexports_callable_values_and_task_calls() {
    let cwd = create_temp_dir("alias-facade");
    let project = cwd.join("app.fpasprj");
    support::write_program_project_file(&project, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/model.fpas"),
        "unit App.Model;
        public type Message = enum Empty; Number(Value: integer); end enum;
        end unit;",
    );
    write_text(
        &cwd.join("src/facade.fpas"),
        "unit App.Facade;
        uses App.Model as Model;
        public type Message = Model.Message;
        public function Twice(Value: integer): integer;
        begin return Value * 2; end function;
        end unit;",
    );
    write_text(
        &cwd.join("src/main.fpas"),
        r#"program Main;
        uses App.Facade as Api;
        uses Std.Math as Numbers;
        uses Std.Tasks as Tasks;
        uses Std.Console as Console;
        begin
          const F: function(Value: integer): integer := aPi.Twice;
          const Job: task := go Api.Twice(F(3));
          const Builtin: task := go Numbers.Abs(-2);
          case Api.Message.Number(Tasks.Wait(Job) + Tasks.Wait(Builtin)) of
            when Api.Message.Empty: panic('wrong variant');
            when Api.Message.Number(const Value): Console.WriteLn(Value);
          end case;
          case Api.Message.Empty of
            when Api.Message.Empty: null;
            when Api.Message.Number(_): panic('wrong empty variant');
          end case;
        end program;"#,
    );
    let (exit, stdout, stderr) = support::run_cli_and_capture_output(&project, &cwd);
    assert_eq!(exit, 0, "{stderr}");
    assert_eq!(stdout, "14\n");
    for invalid in [
        "Message.Empty",
        "Model.Message.Empty",
        "App.Model.Message.Empty",
        "App.Facade.Message.Empty",
        "Twice(1)",
    ] {
        write_text(
            &cwd.join("src/main.fpas"),
            &format!(
                r#"program Main; uses App.Facade as Api; begin const Value: Api.Message := {invalid}; end program;"#
            ),
        );
        let (exit, _, stderr) = support::run_cli_args_and_capture_output(
            &["check".to_string(), project.to_string_lossy().into_owned()],
            &cwd,
        );
        assert_ne!(exit, 0, "{invalid}");
        assert!(stderr.contains("F2003"), "{invalid}: {stderr}");
        assert!(!stderr.contains("F9001"), "{invalid}: {stderr}");
    }
    fs::remove_dir_all(&cwd).expect("remove fixture");
}
