//! Project-level rejection of the removed `property` record member.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (FP2018)

use super::*;

#[test]
fn run_cli_rejects_property_in_imported_unit_with_accessor_hint() {
    let cwd = create_temp_dir("run-removed-property-unit");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;
uses App.Data, Std.Console, Std.Conv;
begin
  WriteLn(IntToStr(Global.GetValue()));
end.",
    );
    write_text(
        &cwd.join("src/data.fpas"),
        "unit App.Data;
public type
  Counter = record
    public Base: integer;
    public function GetValue(Self: Counter): integer;
    begin
      return Self.Base;
    end function;
    public property Value: integer read GetValue;
  end record;
public const Global: Counter := Counter( Base := 12 );
end unit;\n",
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_ne!(exit_code, 0);
    assert!(stderr_output.contains("error[FP2018]"), "{stderr_output}");
    assert!(
        stderr_output.contains("`Value.GetValue()`"),
        "{stderr_output}"
    );
}
