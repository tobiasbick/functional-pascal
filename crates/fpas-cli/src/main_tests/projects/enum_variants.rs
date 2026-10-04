//! Imported enum construction inside independently compiled library code.
//!
//! **Documentation:** `docs/pascal/program-structure/projects.md`

use super::*;

#[test]
fn exhaustive_imported_enum_alias_returns_preserve_declared_backing_values() {
    let cwd = create_temp_dir("imported-enum-case-returns");
    let project = cwd.join("app.fpasprj");
    support::write_program_project_file(&project, "main.fpas", &["*.fpas"]);
    write_text(
        &cwd.join("model.fpas"),
        "unit Demo.Model;
        public type State = enum First = 4; Second = 5; Last = 9; end enum;
        end unit;",
    );
    write_text(
        &cwd.join("facade.fpas"),
        "unit Demo.Facade; uses Demo.Model as Model;
        public type State = Model.State;
        public function Next(Value: State): State;
        begin case Value of
          when State.First: return State.Second;
          when State.Second: return State.Last;
          when State.Last: return State.First;
        end case; end function;
        end unit;",
    );
    write_text(
        &cwd.join("main.fpas"),
        "program Main; uses Demo.Facade as Api; uses Std.Console as Console;
        begin
          Console.WriteLn(Api.Next(Api.State.First) = Api.State.Second);
          Console.WriteLn(Api.Next(Api.State.Second) = Api.State.Last);
          Console.WriteLn(Api.Next(Api.State.Last) = Api.State.First);
        end program;",
    );
    for _ in 0..2 {
        let (exit, stdout, stderr) = support::run_cli_and_capture_output(&project, &cwd);
        assert_eq!(
            (exit, stdout.as_str(), stderr.as_str()),
            (0, "true\ntrue\ntrue\n", "")
        );
    }
    fs::remove_dir_all(&cwd).expect("remove fixture");
}

#[test]
fn payload_patterns_resolve_full_local_and_imported_variant_identity() {
    let cwd = create_temp_dir("enum-pattern-identity");
    let project = cwd.join("app.fpasprj");
    support::write_program_project_file(&project, "main.fpas", &["*.fpas"]);
    write_text(
        &cwd.join("model.fpas"),
        "unit Demo.Model;
        public type Choice of (T) = enum Present(Value: T); Missing; end enum;
        public type OtherChoice = enum Present(Value: string); Missing; end enum;
        end unit;",
    );
    write_text(
        &cwd.join("facade.fpas"),
        "unit Demo.Facade; uses Demo.Model as Model;
        public type IntegerChoice = Model.Choice of (integer);
        public type TextChoice = Model.Choice of (string);
        public type OtherChoice = Model.OtherChoice;
        end unit;",
    );
    write_text(
        &cwd.join("main.fpas"),
        "program Main; uses Demo.Facade as Api; uses Std.Console as Console;
        var Item: Api.IntegerChoice := Api.IntegerChoice.Present(42);
        begin case Item of
          when Api.IntegerChoice.Present(const Value): Console.WriteLn(Value + 0);
          when Api.IntegerChoice.Missing: null;
        end case; end program;",
    );
    for _ in 0..2 {
        let (exit, stdout, stderr) = support::run_cli_and_capture_output(&project, &cwd);
        assert_eq!((exit, stdout.as_str(), stderr.as_str()), (0, "42\n", ""));
    }
    for (pattern, code) in [
        ("Api.OtherChoice.Present(const Value)", "F2006"),
        ("Api.TextChoice.Present(const Value)", "F2006"),
        ("MissingQualifier.Present(const Value)", "F2003"),
        ("Demo.Facade.IntegerChoice.Present(const Value)", "F2003"),
    ] {
        write_text(
            &cwd.join("main.fpas"),
            &format!(
                "program Main;\nuses Demo.Facade as Api;\nvar Item: Api.IntegerChoice := Api.IntegerChoice.Present(42);\nbegin case Item of\nwhen {pattern}: null;\nwhen Api.IntegerChoice.Present(const Value): null;\nwhen Api.IntegerChoice.Missing: null;\nend case; end program;"
            ),
        );
        let args =
            ["check", "--diagnostics", "json", &project.to_string_lossy()].map(str::to_owned);
        let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(&args, &cwd);
        assert_eq!(exit, 1, "{pattern}: {stderr}");
        assert!(stdout.is_empty());
        let diagnostics = stderr
            .lines()
            .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("JSON diagnostic"))
            .collect::<Vec<_>>();
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic["code"] == code
                    && diagnostic["location"]["start"]["line"] == 5),
            "{pattern}: {stderr}"
        );
        assert!(!stderr.contains("F9001"), "{pattern}: {stderr}");
    }
    fs::remove_dir_all(&cwd).expect("remove fixture");
}

#[test]
fn enum_payload_checks_nested_named_record_construction() {
    let cwd = create_temp_dir("enum-record-payload");
    let source = cwd.join("main.fpas");
    write_text(
        &source,
        r#"program NestedRecordEnumConstructor;

uses Std.Console as Console;

type Point = record
  X: integer;
  Y: integer;
end record;

type Position = enum
  At(Value: Point);
end enum;

type State = record
  Player: Position;
end record;

function Moved(Current: Point): State;
begin
  return State(Player := Position.At(Point(Y := Current.Y, X := Current.X + 1)));
end function;

begin
  var Initial: Point := Point(X := 1, Y := 7);
  var Outcome: State := Moved(Initial);
  case Outcome.Player of
    when Position.At(const Value):
      Console.WriteLn(Value.X, ',', Value.Y);
  end case;
end program;
"#,
    );
    for command in ["check", "run"] {
        let (code, stdout, stderr) = support::run_cli_args_and_capture_output(
            &[command.to_string(), source.to_string_lossy().into_owned()],
            &cwd,
        );
        assert_eq!(code, 0, "{command}: {stderr}");
        if command == "run" {
            assert_eq!(stdout, "2,7\n");
        }
    }
    fs::remove_dir_all(&cwd).expect("remove fixture");
}

#[test]
fn enum_record_payload_rejects_incompatible_arguments() {
    let cwd = create_temp_dir("invalid-enum-record-payload");
    let source = cwd.join("main.fpas");
    for argument in [
        "Point(X := 'wrong', Y := 2)",
        "Point(X := 1)",
        "Point(X := 1, Y := 2, Z := 3)",
        "Other",
    ] {
        write_text(
            &source,
            &format!(
                "program InvalidEnumRecordPayload;\n\n  type Point = record X: integer; Y: integer; end record;\n  type Size = record X: integer; Y: integer; end record;\n  type Position = enum At(Value: Point); end enum;\nbegin\n  var Other: Size := Size(X := 1, Y := 2);\n  var Value: Position := Position.At({argument});\nend program;"
            ),
        );
        let (code, _, stderr) = support::run_cli_args_and_capture_output(
            &["check".to_string(), source.to_string_lossy().into_owned()],
            &cwd,
        );
        assert_ne!(code, 0, "accepted {argument}");
        assert!(stderr.contains("error[F2"), "{argument}: {stderr}");
        assert!(!stderr.contains("F9001"), "{argument}: {stderr}");
    }
    fs::remove_dir_all(&cwd).expect("remove fixture");
}

#[test]
fn run_cli_library_constructs_imported_json_variant_inside_local_enum_arm() {
    let cwd = create_temp_dir("library-imported-enum-variant");
    let project_file = cwd.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "program"
main = "main.fpas"
[sources]
include = ["main.fpas"]
[dependencies]
projects = ["json-lib.fpasprj"]
"#,
    );
    support::write_library_project_file(&cwd.join("json-lib.fpasprj"), &["json.fpas"]);
    write_text(
        &cwd.join("json.fpas"),
        r#"unit Repro.Json;

uses Std.Json as Json;

public type Message = enum
  ErrorMessage(Code: string);
end enum;

public function Encode(Value: Message): string;
begin
  case Value of
    when Message.ErrorMessage(const Code):
      begin
        return Json.Stringify(Json.JsonValue.String(Code));
      end;
  end case;
end function;

end unit;
"#,
    );
    write_text(
        &cwd.join("main.fpas"),
        r#"program JsonLibraryRepro;
uses Repro.Json as Json; uses Std.Console as Console;
begin
  Console.WriteLn(Json.Encode(Json.Message.ErrorMessage('code')));
end program;"#,
    );

    let (exit_code, stdout, stderr) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("remove fixture");
    assert_eq!(exit_code, 0, "stderr: {stderr}");
    assert_eq!(stdout, "\"code\"\n");
}
