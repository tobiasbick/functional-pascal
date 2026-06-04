use super::*;
use crate::test_support::{write_library_fpasprj, write_program_fpasprj_with_deps};

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
        "unit Calc.Math;\nfunction Mul(A: integer; B: integer): integer;\nbegin\n  return A * B\nend;\n",
    );

    write_program_fpasprj_with_deps(
        &app_project,
        "src/main.fpas",
        &["src/**/*.fpas"],
        &["../../libs/math/math.fpasprj"],
    );
    write_text(
        &app_dir.join("src/main.fpas"),
        "program Calc;\nuses Calc.Math, Std.Console;\nbegin\n  WriteLn(Mul(6, 7))\nend.\n",
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&app_project, &app_dir);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "42\n");
    assert!(stderr_output.is_empty());
}
