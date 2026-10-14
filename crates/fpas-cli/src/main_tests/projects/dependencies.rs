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
        "unit Calc.Math;\npublic function Mul(A: integer; B: integer): integer;\nbegin\n  return A * B;\nend function;\nend unit;\n",
    );

    write_program_fpasprj_with_deps(
        &app_project,
        "src/main.fpas",
        &["src/**/*.fpas"],
        &["../../libs/math/math.fpasprj"],
    );
    write_text(
        &app_dir.join("src/main.fpas"),
        "program Calc;\nuses Calc.Math as Math, Std.Console as Console;\nbegin\n  Console.WriteLn(Math.Mul(6, 7));\nend.\n",
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
        "unit Geom.Internal;\n\nuses Std.Console as Console;\n\npublic type\n  PointImpl = record\n    public X: integer;\n    public Y: integer;\n\n    public static function Create(X: integer; Y: integer): PointImpl;\n    begin\n      return PointImpl(\n        X := X,\n        Y := Y\n      );\n    end function;\n\n    public static procedure Print(Value: PointImpl);\n    begin\n      Console.WriteLn(Value.Sum());\n    end procedure;\n\n    public function Sum(Self: PointImpl): integer;\n    begin\n      return Self.X + Self.Y;\n    end function;\n  end record;\nend unit;\n",
    );
    write_text(
        &lib_dir.join("src/api.fpas"),
        "unit Geom.Api;\n\nuses Geom.Internal as Internal;\n\npublic type\n  Point = Internal.PointImpl;\nend unit;\n",
    );

    write_program_fpasprj_with_deps(
        &app_project,
        "src/main.fpas",
        &["src/**/*.fpas"],
        &["../geom/geom.fpasprj"],
    );
    write_text(
        &app_dir.join("src/main.fpas"),
        "program App;\nuses Geom.Api as Api;\nbegin\n  const P: Api.Point := Api.Point.Create(3, 4);\n  Api.Point.Print(P);\nend.\n",
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&app_project, &app_dir);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "7\n");
    assert!(stderr_output.is_empty());
}
