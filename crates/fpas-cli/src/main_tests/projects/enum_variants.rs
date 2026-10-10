//! Imported enum construction inside independently compiled library code.
//!
//! **Documentation:** `docs/pascal/program-structure/projects.md`

use super::*;

#[test]
fn enum_payload_infers_nested_anonymous_record_type() {
    let cwd = create_temp_dir("enum-record-payload");
    let source = cwd.join("main.fpas");
    write_text(
        &source,
        "program NestedRecordEnumConstructor;
uses Std.Console;
type
  Point = record X: integer; Y: integer; end record;
  type Position = enum At(Value: Point); end enum;
  type State = record Player: Position; end record;
function Moved(Current: Point): State;
begin
  return State(
    Player := Position.At(Point(
      X := Current.X + 1,
      Y := Current.Y
    ))\n  );
end function;
begin
  const Initial: Point := Point( X := 1, Y := 7 );
  const Outcome: State := Moved(Initial);
  case Outcome.Player of
    when Position.At(const Value): WriteLn(Value.X, ',', Value.Y);
  end case;
end.",
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
        "Point( X := 'wrong', Y := 2 )",
        "Point( X := 1 )",
        "Point( X := 1, Y := 2, Z := 3 )",
        "Other",
    ] {
        write_text(
            &source,
            &format!(
                "program InvalidEnumRecordPayload;
type
  Point = record X: integer; Y: integer; end record;
  type Size = record X: integer; Y: integer; end record;
  type Position = enum At(Value: Point); end enum;
begin
  const Other: Size := Size( X := 1, Y := 2 );
  const Value: Position := Position.At({argument});
end."
            ),
        );
        let (code, _, stderr) = support::run_cli_args_and_capture_output(
            &["check".to_string(), source.to_string_lossy().into_owned()],
            &cwd,
        );
        assert_ne!(code, 0, "accepted {argument}");
        assert!(stderr.contains("error[FP3"), "{argument}: {stderr}");
        assert!(!stderr.contains("FP9001"), "{argument}: {stderr}");
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
        "unit Repro.Json;
uses Std.Json;
public type
  Message = enum
    ErrorMessage(Code: string);
  end enum;
public function Encode(Value: Message): string;
begin
  case Value of
    when Message.ErrorMessage(const Code):
    begin
      return Stringify(JsonValue.String(Code));
    end;
  end case;
end function;\nend unit;",
    );
    write_text(
        &cwd.join("main.fpas"),
        "program JsonLibraryRepro;
uses Repro.Json, Std.Console;
begin
  WriteLn(Encode(Message.ErrorMessage('code')));
end.",
    );

    let (exit_code, stdout, stderr) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("remove fixture");
    assert_eq!(exit_code, 0, "stderr: {stderr}");
    assert_eq!(stdout, "\"code\"\n");
}

#[test]
fn pattern_diagnostics_reject_duplicate_values_and_invalid_qualifiers() {
    let cwd = create_temp_dir("pattern-resolution-diagnostics");
    let source = cwd.join("main.fpas");
    for (body, expected_code) in [
        (
            "case Value of when Some(Shape.Circle(1)): null; when Some(Shape.Circle(1)): null; when Some(_), None: null; end case;",
            "FP3033",
        ),
        (
            "case Value of when Some(Other.Point): null; when Some(_), None: null; end case;",
            "FP3006",
        ),
        (
            "if Value is Some(Other.Circle(_)) then null; end if;",
            "FP3006",
        ),
        (
            "if Value is Some(Missing.Point) then null; end if;",
            "FP3003",
        ),
        (
            "if Value is Some(Shape.Circle(Radius)) then null; end if;",
            "FP3031",
        ),
    ] {
        write_text(
            &source,
            &format!(
                "program PatternResolution;
type Shape = enum Circle(Radius: integer); Point; end enum;
type Other = enum Circle(Radius: integer); Point; end enum;
procedure Probe(Value: option of Shape);
begin {body} end procedure;
begin end."
            ),
        );
        for command in ["check", "run"] {
            let (code, _, stderr) = support::run_cli_args_and_capture_output(
                &[command.to_string(), source.to_string_lossy().into_owned()],
                &cwd,
            );
            assert_ne!(code, 0, "{command} accepted {body}");
            assert!(stderr.contains(expected_code), "{command}: {stderr}");
            assert!(!stderr.contains("FP9001"), "{command}: {stderr}");
        }
    }
    fs::remove_dir_all(&cwd).expect("remove fixture");
}
