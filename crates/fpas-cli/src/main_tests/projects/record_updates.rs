use super::*;

#[test]
fn imported_record_update_literals_run_without_mutating_the_original() {
    let cwd = create_temp_dir("record-update-imported-literals");
    let project = cwd.join("app.fpasprj");
    write_text(
        &project,
        r#"[project]
name = "app"
kind = "program"
main = "main.fpas"
[sources]
include = ["*.fpas"]
"#,
    );
    write_text(
        &cwd.join("types.fpas"),
        r#"unit Demo.Types;
  public type Point = record public X: integer; public Y: integer; end record;
  public type Holder = record public Position: Point; public Points: array of Point; end record;
end unit;
"#,
    );
    write_text(
        &cwd.join("main.fpas"),
        r#"program Main;
uses Demo.Types as Types; uses Std.Console as Console;
begin
  var Original: Types.Holder := record
    Position := record X := 1; Y := 2; end record;
    Points := [record X := 5; Y := 6; end record];
  end record;
  var Changed: Types.Holder := Original with
    Position := record X := 3; Y := 4; end record;
    Points := [record X := 7; Y := 8; end record];
  end with;
  Console.WriteLn(Changed.Position.X);
  Console.WriteLn(Changed.Position.Y);
  Console.WriteLn(Changed.Points[0].X);
  Console.WriteLn(Changed.Points[0].Y);
  Console.WriteLn(Original.Position.X);
  Console.WriteLn(Original.Position.Y);
  Console.WriteLn(Original.Points[0].X);
  Console.WriteLn(Original.Points[0].Y);
end program;"#,
    );
    let (exit_code, stdout, stderr) =
        super::super::support::run_cli_and_capture_output(&project, &cwd);
    fs::remove_dir_all(&cwd).expect("remove temporary project");
    assert_eq!(exit_code, 0, "{stderr}");
    assert!(stderr.is_empty(), "{stderr}");
    assert_eq!(stdout, "3\n4\n7\n8\n1\n2\n5\n6\n");
}
