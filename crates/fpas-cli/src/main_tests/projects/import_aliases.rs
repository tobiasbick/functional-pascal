//! Source alias identities across enum facades, callable values, and tasks.
//!
//! **Documentation:** `docs/pascal/program-structure/units.md`.

use super::*;

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
        "program Main;
        uses App.Facade as Api;
        uses Std.Math as Numbers;
        uses Std.Tasks as Tasks;
        uses Std.Console as Console;
        begin
          var F: function(Value: integer): integer := aPi.Twice;
          var Job: task := go Api.Twice(F(3));
          var Builtin: task := go Numbers.Abs(-2);
          case Api.Message.Number(Tasks.Wait(Job) + Tasks.Wait(Builtin)) of
            when Api.Message.Empty: panic('wrong variant');
            when Api.Message.Number(const Value): Console.WriteLn(Value);
          end case;
          case Api.Message.Empty of
            when Api.Message.Empty: null;
            when Api.Message.Number(_): panic('wrong empty variant');
          end case;
        end program;",
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
                "program Main; uses App.Facade as Api; begin var Value: Api.Message := {invalid}; end program;"
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
