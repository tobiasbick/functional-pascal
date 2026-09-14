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
  Point = record X: integer; Y: integer; end;
  Position = enum At(Value: Point); end;
  State = record Player: Position; end;
function Moved(Current: Point): State;
begin
  return record
    Player := Position.At(record
      Y := Current.Y;
      X := Current.X + 1;
    end);
  end
end;
begin
  var Initial: Point := record X := 1; Y := 7; end;
  var Outcome: State := Moved(Initial);
  case Outcome.Player of
    Position.At(Value): WriteLn(Value.X, ',', Value.Y)
  end
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
        "record X := 'wrong'; Y := 2; end",
        "record X := 1; end",
        "record X := 1; Y := 2; Z := 3; end",
        "Other",
    ] {
        write_text(
            &source,
            &format!(
                "program InvalidEnumRecordPayload;
type
  Point = record X: integer; Y: integer; end;
  Size = record X: integer; Y: integer; end;
  Position = enum At(Value: Point); end;
begin
  var Other: Size := record X := 1; Y := 2; end;
  var Value: Position := Position.At({argument})
end."
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
        "unit Repro.Json;
uses Std.Json;
public type
  Message = enum
    ErrorMessage(Code: string);
  end;
public function Encode(Value: Message): string;
begin
  case Value of
    Message.ErrorMessage(Code):
    begin
      return Stringify(JsonValue.String(Code))
    end
  end
end;",
    );
    write_text(
        &cwd.join("main.fpas"),
        "program JsonLibraryRepro;
uses Repro.Json, Std.Console;
begin
  WriteLn(Encode(Message.ErrorMessage('code')))
end.",
    );

    let (exit_code, stdout, stderr) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("remove fixture");
    assert_eq!(exit_code, 0, "stderr: {stderr}");
    assert_eq!(stdout, "\"code\"\n");
}
