//! Imported record data uses ordinary functions and explicit closures.

use super::*;

#[test]
fn run_cli_captures_unit_global_record_in_an_ordinary_closure() {
    let cwd = create_temp_dir("run-record-closure-unit-global");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        r#"program Main;
uses App.Data as Data; uses Std.Console as Console; uses Std.Conv as Conv;
begin
  const Snapshot := Data.Global;
  const AddTwelve := function(Value: integer): integer
    begin return Data.CounterAdd(Snapshot, Value); end function;
  Console.WriteLn(Conv.IntToStr(AddTwelve(3)));
end program;"#,
    );
    write_text(
        &cwd.join("src/data.fpas"),
        r#"unit App.Data;

public type Counter = record
  public Base: integer;
end record;

  public function CounterAdd(Receiver: Counter; Value: integer): integer;
  begin
    return Receiver.Base + Value;
  end function;

public const Global: Counter := Counter(Base := 12);

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

#[test]
fn run_cli_calls_accessors_with_unit_global_record() {
    let cwd = create_temp_dir("run-record-accessor-unit-global");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        r#"program Main;
uses App.Data as Data; uses Std.Console as Console; uses Std.Conv as Conv;
begin
  Console.WriteLn(Conv.IntToStr(Data.CounterGetValue(Data.Global)));
  Data.CounterSetValue(Data.Global, 19);
end program;"#,
    );
    write_text(
        &cwd.join("src/data.fpas"),
        r#"unit App.Data;

uses Std.Console as Console;
uses Std.Conv as Conv;

public type Counter = record
  Base: integer;
end record;

  public function CounterGetValue(Receiver: Counter): integer;
  begin
    return Receiver.Base;
  end function;

  public procedure CounterSetValue(Receiver: Counter; Value: integer);
  begin
    Console.WriteLn('set:' + Conv.IntToStr(Value));
  end procedure;


public const Global: Counter := Counter(Base := 12);

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

#[test]
fn run_cli_assigns_optional_handler_and_calls_it_through_owner_unit() {
    let cwd = create_temp_dir("run-optional-handler-owner-unit");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        r#"program Main;
uses App.Widget as Widget; uses Std.Console as Console; uses Std.Conv as Conv;
procedure Handle(Value: integer);
begin
  Console.WriteLn(Conv.IntToStr(Value));
end procedure;
begin
  var B := Widget.ButtonMake(14);
  Widget.ButtonClick(B);
  B.OnClick := Option.Some(Handle);
  Widget.ButtonClick(B);
  B.OnClick := Option.None;
  Widget.ButtonClick(B);
end program;"#,
    );
    write_text(
        &cwd.join("src/widget.fpas"),
        r#"unit App.Widget;

public type Button = record
  Id: integer;
  public OnClick: Option of (procedure(Value: integer)) := Option.None;
end record;

  public procedure ButtonClick(Receiver: Button);
  begin
    case Receiver.OnClick of
      when Option.Some(const Handler): Handler(Receiver.Id);
      when Option.None: null;
    end case;
  end procedure;

  public function ButtonMake(Id: integer): Button;
  begin
    return Button(Id := Id);
  end function;


end unit;
"#,
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "14\n");
    assert!(stderr_output.is_empty());
}
