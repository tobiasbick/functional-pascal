//! Project-linking regression tests for bound record methods.
//!
//! **Documentation:** `docs/pascal/language/types/record-methods.md`

use super::*;

#[test]
fn run_cli_binds_method_from_unit_global_record() {
    let cwd = create_temp_dir("run-bound-method-unit-global");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        r#"program Main;
uses App.Data as Data; uses Std.Console as Console; uses Std.Conv as Conv;
begin
  var AddTwelve: function(Value: integer): integer := Data.Global.Add;
  Console.WriteLn(Conv.IntToStr(AddTwelve(3)));
end program;"#,
    );
    write_text(
        &cwd.join("src/data.fpas"),
        r#"unit App.Data;

  public type Counter = record
    public Base: integer;
    public function Add(Self: Counter; Value: integer): integer;
    begin
      return Self.Base + Value;
    end function;
  end record;
  public var Global: Counter := record Base := 12; end record;
end unit;

"#,
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "15\n");
    assert!(stderr_output.is_empty());
}
