//! Project-linking regression tests for record properties.
//!
//! **Documentation:** `docs/pascal/language/types/record-properties.md`

use super::*;

#[test]
fn run_cli_reads_and_writes_property_from_unit_global_record() {
    let cwd = create_temp_dir("run-record-property-unit-global");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;
uses App.Data, Std.Console, Std.Conv;
begin
  WriteLn(IntToStr(Global.Value));
  Global.Value := 19
end.",
    );
    write_text(
        &cwd.join("src/data.fpas"),
        "unit App.Data;
uses Std.Console, Std.Conv;
public type
  Counter = record
    public Base: integer;
    public function GetValue(Self: Counter): integer;
    begin
      return Self.Base
    end;
    public procedure SetValue(Self: Counter; Value: integer);
    begin
      WriteLn('set:' + IntToStr(Value))
    end;
    public property Value: integer read GetValue write SetValue;
  end;
public var Global: Counter := record Base := 12; end;
",
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "12\nset:19\n");
    assert!(stderr_output.is_empty());
}
