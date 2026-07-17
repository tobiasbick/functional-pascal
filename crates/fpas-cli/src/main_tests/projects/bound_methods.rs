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
        "program Main;
uses App.Data, Std.Console, Std.Conv;
begin
  var AddTwelve: function(Value: integer): integer := Global.Add;
  WriteLn(IntToStr(AddTwelve(3)))
end.",
    );
    write_text(
        &cwd.join("src/data.fpas"),
        "unit App.Data;
type
  Counter = record
    Base: integer;
    function Add(Self: Counter; Value: integer): integer;
    begin
      return Self.Base + Value
    end;
  end;
var Global: Counter := record Base := 12; end;
",
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "15\n");
    assert!(stderr_output.is_empty());
}
