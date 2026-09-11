//! Imported enum construction inside independently compiled library code.
//!
//! **Documentation:** `docs/pascal/program-structure/projects.md`

use super::*;

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
