//! Project-linking regression tests for record events.
//!
//! **Documentation:** `docs/pascal/language/types/record-events.md`

use super::*;

#[test]
fn run_cli_assigns_and_owner_unit_raises_record_event() {
    let cwd = create_temp_dir("run-record-event-owner-unit");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;
uses App.Widget, Std.Console, Std.Conv;
procedure Handle(Value: integer);
begin
  WriteLn(IntToStr(Value));
end procedure;
begin
  const B: Button := Button.Make(14);
  B.OnClick := Handle;
  if Assigned(B.OnClick) then
    B.Click(); end if;
  B.OnClick := nil;
end.",
    );
    write_text(
        &cwd.join("src/widget.fpas"),
        "unit App.Widget;
public var Slot: Option of procedure(Value: integer) := None;
public type
  Button = record
    public Id: integer;
    public function ReadOnClick(Self: Button): Option of procedure(Value: integer);
    begin
      return Slot;
    end function;
    public procedure WriteOnClick(Self: Button; Handler: Option of procedure(Value: integer));
    begin
      Slot := Handler;
    end procedure;
    public event OnClick: procedure(Value: integer) read ReadOnClick write WriteOnClick;
    public procedure Click(Self: Button);
    begin
      if Assigned(Self.OnClick) then
        Self.OnClick(Self.Id); end if;
    end procedure;
    public static function Make(Id: integer): Button;
    begin
      return Button( Id := Id );
    end function;
  end record;
end unit;\n",
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "14\n");
    assert!(stderr_output.is_empty());
}
