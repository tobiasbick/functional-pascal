use super::*;
use crate::test_support::{
    write_library_fpasprj, write_library_fpasprj_with_exports, write_program_fpasprj_with_deps,
};

#[test]
fn run_cli_executes_program_with_library_project_dependency() {
    let cwd = create_temp_dir("run-project-library-dep");
    let lib_dir = cwd.join("libs").join("math");
    let app_dir = cwd.join("apps").join("calc");
    let lib_project = lib_dir.join("math.fpasprj");
    let app_project = app_dir.join("calc.fpasprj");

    write_library_fpasprj(&lib_project, &["src/**/*.fpas"]);
    write_text(
        &lib_dir.join("src/math.fpas"),
        r#"unit Calc.Math;
public function Mul(A: integer; B: integer): integer;
begin
  return A * B;
end function;
end unit;

"#,
    );

    write_program_fpasprj_with_deps(
        &app_project,
        "src/main.fpas",
        &["src/**/*.fpas"],
        &["../../libs/math/math.fpasprj"],
    );
    write_text(
        &app_dir.join("src/main.fpas"),
        r#"program Calc;
uses Calc.Math as Math; uses Std.Console as Console;
begin
  Console.WriteLn(Math.Mul(6, 7));
end program;
"#,
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&app_project, &app_dir);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "42\n");
    assert!(stderr_output.is_empty());
}

#[test]
fn run_cli_static_record_function_via_public_alias_over_private_unit() {
    let cwd = create_temp_dir("run-static-alias-facade");
    let lib_dir = cwd.join("geom");
    let app_dir = cwd.join("app");
    let lib_project = lib_dir.join("geom.fpasprj");
    let app_project = app_dir.join("app.fpasprj");

    write_library_fpasprj_with_exports(&lib_project, &["src/**/*.fpas"], &["Geom.Api"]);
    write_text(
        &lib_dir.join("src/internal.fpas"),
        r#"unit Geom.Internal;

uses Std.Console as Console;

public type PointImpl = record
  public X: integer;
  public Y: integer;

  public static function Create(X: integer; Y: integer): PointImpl;
  begin
    return PointImpl(X := X, Y := Y);
  end function;

  public static procedure Print(Value: PointImpl);
  begin
    Console.WriteLn(Value.Sum());
  end procedure;

  public function Sum(Self: PointImpl): integer;
  begin
    return Self.X + Self.Y;
  end function;
end record;

end unit;
"#,
    );
    write_text(
        &lib_dir.join("src/api.fpas"),
        r#"unit Geom.Api;

uses Geom.Internal as Internal;


  public type Point = Internal.PointImpl;
end unit;

"#,
    );

    write_program_fpasprj_with_deps(
        &app_project,
        "src/main.fpas",
        &["src/**/*.fpas"],
        &["../geom/geom.fpasprj"],
    );
    write_text(
        &app_dir.join("src/main.fpas"),
        r#"program App;
uses Geom.Api as Api;
begin
  var P: Api.Point := Api.Point.Create(3, 4);
  Api.Point.Print(P);
end program;
"#,
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&app_project, &app_dir);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "7\n");
    assert!(stderr_output.is_empty());
}
