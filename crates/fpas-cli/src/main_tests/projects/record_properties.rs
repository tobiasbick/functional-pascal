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
        r#"program Main;
uses App.Data as Data; uses Std.Console as Console; uses Std.Conv as Conv;
begin
  Console.WriteLn(Conv.IntToStr(Data.Global.Value));
  Data.Global.Value := 19;
end program;"#,
    );
    write_text(
        &cwd.join("src/data.fpas"),
        r#"unit App.Data;

uses Std.Console as Console;
uses Std.Conv as Conv;

public type Counter = record
  public Base: integer;

  public function GetValue(Self: Counter): integer;
  begin
    return Self.Base;
  end function;

  public procedure SetValue(Self: Counter; Value: integer);
  begin
    Console.WriteLn('set:' + Conv.IntToStr(Value));
  end procedure;

  public property Value: integer read GetValue write SetValue;
end record;

public var Global: Counter := Counter(Base := 12);

end unit;
"#,
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "12\nset:19\n");
    assert!(stderr_output.is_empty());
}
