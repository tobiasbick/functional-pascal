use super::*;

#[test]
fn run_cli_rejects_library_projects() {
    let cwd = create_temp_dir("run-library-project");
    let project_file = cwd.join("lib.fpasprj");
    support::write_library_project_file(&project_file, &["src/**/*.fpas"]);
    write_text(&cwd.join("src/util.fpas"), "unit Lib.Util;\nend unit;");

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(stderr_output.contains("Library projects are not executable"));
    assert!(!stderr_output.contains("warning:"));
}

#[test]
fn run_cli_reports_cyclic_unit_dependencies() {
    let cwd = create_temp_dir("run-cycle");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses App.A;\nbegin\nend.\n",
    );
    write_text(
        &cwd.join("src/a.fpas"),
        "unit App.A;\nuses App.B;\nend unit;\n",
    );
    write_text(
        &cwd.join("src/b.fpas"),
        "unit App.B;\nuses App.A;\nend unit;\n",
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(stderr_output.contains("Cyclic unit dependency detected"));
}

#[test]
fn run_cli_reports_unknown_user_unit() {
    let cwd = create_temp_dir("run-unknown-unit");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses App.Missing;\nbegin\nend.\n",
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(stderr_output.contains("Unknown unit `App.Missing`"));
}

#[test]
fn run_cli_reports_ambiguous_user_imports() {
    let cwd = create_temp_dir("run-ambiguous-import");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses App.Math, App.Advanced;\nbegin\n  Add(1, 2);\nend.\n",
    );
    write_text(
        &cwd.join("src/math.fpas"),
        "unit App.Math;\npublic function Add(A: integer; B: integer): integer;\nbegin\n  return A + B;\nend function;\nend unit;\n",
    );
    write_text(
        &cwd.join("src/advanced.fpas"),
        "unit App.Advanced;\npublic function Add(A: integer; B: integer): integer;\nbegin\n  return A - B;\nend function;\nend unit;\n",
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(stderr_output.contains("Ambiguous imported symbol `Add`"));
}

#[test]
fn run_cli_reports_unit_sema_errors_with_the_unit_path() {
    let cwd = create_temp_dir("run-unit-sema-path");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses App.Util;\nbegin\nend.\n",
    );
    write_text(
        &cwd.join("src/util.fpas"),
        "unit App.Util;\npublic function Broken(): integer;\nbegin\n  return Missing;\nend function;\nend unit;\n",
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("util.fpas:4:10: error[FP3003]: Undefined identifier `Missing`")
    );
    assert!(!stderr_output.contains("main.fpas:4:10: error[FP3003]"));
}

#[test]
fn run_cli_reports_unit_runtime_errors_with_the_unit_path() {
    let cwd = create_temp_dir("run-unit-runtime-path");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses App.Util;\nbegin\n  Trigger();\nend.\n",
    );
    write_text(
        &cwd.join("src/util.fpas"),
        "unit App.Util;\npublic procedure Trigger();\nbegin\n  const X: integer := 1 div 0;\nend procedure;\nend unit;\n",
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 2);
    assert!(stderr_output.contains("util.fpas:4:23: error[FP5001]: Division by zero"));
    assert!(!stderr_output.contains("main.fpas:4:23: error[FP5001]"));
}

#[test]
fn run_cli_reports_runtime_errors_of_units_linked_out_of_graph_order() {
    let cwd = create_temp_dir("run-unit-link-order-path");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses App.Util;\nbegin\n  Trigger();\nend.\n",
    );
    // `util.fpas` precedes `zeta.fpas` in the unit graph, but the linker emits App.Zeta first.
    write_text(
        &cwd.join("src/util.fpas"),
        "unit App.Util;\nuses App.Zeta;\npublic procedure Trigger();\nbegin\n  if Seven() = 7 then panic('util failure'); end if;\nend procedure;\nend unit;\n",
    );
    write_text(
        &cwd.join("src/zeta.fpas"),
        "unit App.Zeta;\npublic function Seven(): integer;\nbegin\n  return 7;\nend function;\nend unit;\n",
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 2);
    assert!(
        stderr_output.contains("util.fpas:5:"),
        "panic must name the unit that raised it: {stderr_output}"
    );
    assert!(stderr_output.contains("util failure"));
    assert!(!stderr_output.contains("zeta.fpas:"));
}
