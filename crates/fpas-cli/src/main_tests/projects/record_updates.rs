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
public type Point = record public X: integer; public Y: integer; end record;
public type Holder = record public Position: Point; public Points: array of Point; end record;\nend unit;",
    );
    write_text(
        &cwd.join("main.fpas"),
        "program Main;\nuses Demo.Types as Types, Std.Console as Console;\nbegin\n  const Original: Types.Holder := Types.Holder(\n    Position := Types.Point( X := 1, Y := 2 ),\n    Points := [Types.Point( X := 5, Y := 6 )]\n  );\n  const Changed: Types.Holder := Original with\n    Position := Types.Point( X := 3, Y := 4 );\n    Points := [Types.Point( X := 7, Y := 8 )];\n  end with;\n  Console.WriteLn(Changed.Position.X);\n  Console.WriteLn(Changed.Position.Y);\n  Console.WriteLn(Changed.Points[0].X);\n  Console.WriteLn(Changed.Points[0].Y);\n  Console.WriteLn(Original.Position.X);\n  Console.WriteLn(Original.Position.Y);\n  Console.WriteLn(Original.Points[0].X);\n  Console.WriteLn(Original.Points[0].Y);\nend.",
    );
    let (exit_code, stdout, stderr) =
        super::super::support::run_cli_and_capture_output(&project, &cwd);
    fs::remove_dir_all(&cwd).expect("remove temporary project");
    assert_eq!(exit_code, 0, "{stderr}");
    assert!(stderr.is_empty(), "{stderr}");
    assert_eq!(stdout, "3\n4\n7\n8\n1\n2\n5\n6\n");
}
