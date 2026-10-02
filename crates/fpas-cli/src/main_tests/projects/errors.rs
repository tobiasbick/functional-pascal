use super::*;

#[test]
fn run_cli_rejects_library_projects() {
    let cwd = create_temp_dir("run-library-project");
    let project_file = cwd.join("lib.fpasprj");
    support::write_library_project_file(&project_file, &["src/**/*.fpas"]);
    write_text(
        &cwd.join("src/util.fpas"),
        r#"unit Lib.Util;
end unit;
"#,
    );

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
        r#"program Main;
uses App.A as A;
begin null;
end program;
"#,
    );
    write_text(
        &cwd.join("src/a.fpas"),
        r#"unit App.A;
uses App.B as B;
end unit;

"#,
    );
    write_text(
        &cwd.join("src/b.fpas"),
        r#"unit App.B;
uses App.A as A;
end unit;

"#,
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
        r#"program Main;
uses App.Missing as Missing;
begin null;
end program;
"#,
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(stderr_output.contains("Unknown unit `App.Missing`"));
}

#[test]
fn run_cli_reports_unqualified_user_imports() {
    let cwd = create_temp_dir("run-ambiguous-import");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        r#"program Main;
uses App.Math as Math; uses App.Advanced as Advanced;
begin
  Add(1, 2);
end program;
"#,
    );
    write_text(
        &cwd.join("src/math.fpas"),
        r#"unit App.Math;
public function Add(A: integer; B: integer): integer;
begin
  return A + B;
end function;
end unit;

"#,
    );
    write_text(
        &cwd.join("src/advanced.fpas"),
        r#"unit App.Advanced;
public function Add(A: integer; B: integer): integer;
begin
  return A - B;
end function;
end unit;

"#,
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("Unknown procedure `Add`"),
        "{stderr_output}"
    );
    assert!(
        stderr_output.to_ascii_lowercase().contains("math.add")
            && stderr_output.to_ascii_lowercase().contains("advanced.add")
    );
}

#[test]
fn run_cli_reports_unit_sema_errors_with_the_unit_path() {
    let cwd = create_temp_dir("run-unit-sema-path");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        r#"program Main;
uses App.Util as Util;
begin null;
end program;
"#,
    );
    write_text(
        &cwd.join("src/util.fpas"),
        r#"unit App.Util;
public function Broken(): integer;
begin
  return Missing;
end function;
end unit;

"#,
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(stderr_output.contains("util.fpas:4:10: error[F2003]: Undefined identifier `Missing`"));
    assert!(!stderr_output.contains("main.fpas:4:10: error[F2003]"));
}

#[test]
fn run_cli_reports_unit_runtime_errors_with_the_unit_path() {
    let cwd = create_temp_dir("run-unit-runtime-path");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        r#"program Main;
uses App.Util as Util;
begin
  Util.Trigger();
end program;
"#,
    );
    write_text(
        &cwd.join("src/util.fpas"),
        r#"unit App.Util;
public procedure Trigger();
begin
  var X: integer := 1 div 0;
end procedure;
end unit;

"#,
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 2);
    assert!(stderr_output.contains("util.fpas:4:21: error[F4001]: Division by zero"));
    assert!(!stderr_output.contains("main.fpas:4:21: error[F4001]"));
}

#[test]
fn run_cli_reports_runtime_errors_of_units_linked_out_of_graph_order() {
    let cwd = create_temp_dir("run-unit-link-order-path");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        r#"program Main;
uses App.Util as Util;
begin
  Util.Trigger();
end program;
"#,
    );
    // `util.fpas` precedes `zeta.fpas` in the unit graph, but the linker emits App.Zeta first.
    write_text(
        &cwd.join("src/util.fpas"),
        r#"unit App.Util;
uses App.Zeta as Zeta;
public procedure Trigger();
begin
  if Zeta.Seven() = 7 then panic('util failure'); end if;
end procedure;
end unit;

"#,
    );
    write_text(
        &cwd.join("src/zeta.fpas"),
        r#"unit App.Zeta;
public function Seven(): integer;
begin
  return 7;
end function;
end unit;

"#,
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
