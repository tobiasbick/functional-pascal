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
        "unit Demo.Types;
public type Point = record public X: integer; public Y: integer; end;
public type Holder = record public Position: Point; public Points: array of Point; end;",
    );
    write_text(
        &cwd.join("main.fpas"),
        "program Main;
uses Demo.Types, Std.Console;
begin
  var Original: Holder := record
    Position := record X := 1; Y := 2; end;
    Points := [record X := 5; Y := 6; end];
  end;
  var Changed: Holder := Original with
    Position := record X := 3; Y := 4; end;
    Points := [record X := 7; Y := 8; end];
  end;
  WriteLn(Changed.Position.X);
  WriteLn(Changed.Position.Y);
  WriteLn(Changed.Points[0].X);
  WriteLn(Changed.Points[0].Y);
  WriteLn(Original.Position.X);
  WriteLn(Original.Position.Y);
  WriteLn(Original.Points[0].X);
  WriteLn(Original.Points[0].Y);
end.",
    );
    let (exit_code, stdout, stderr) =
        super::super::support::run_cli_and_capture_output(&project, &cwd);
    fs::remove_dir_all(&cwd).expect("remove temporary project");
    assert_eq!(exit_code, 0, "{stderr}");
    assert!(stderr.is_empty(), "{stderr}");
    assert_eq!(stdout, "3\n4\n7\n8\n1\n2\n5\n6\n");
}
