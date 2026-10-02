use super::*;

#[test]
fn diamond_dependency_graph() {
    let cwd = create_temp_dir("run-diamond-deps");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        r#"program Main;
uses App.A as A; uses App.B as B; uses Std.Console as Console;
begin
  Console.WriteLn(A.FromA() + B.FromB());
end program;
"#,
    );
    write_text(
        &cwd.join("src/a.fpas"),
        r#"unit App.A;
uses App.Shared as Shared;
public function FromA(): integer;
begin
  return Shared.Base() + 1;
end function;
end unit;

"#,
    );
    write_text(
        &cwd.join("src/b.fpas"),
        r#"unit App.B;
uses App.Shared as Shared;
public function FromB(): integer;
begin
  return Shared.Base() + 10;
end function;
end unit;

"#,
    );
    write_text(
        &cwd.join("src/shared.fpas"),
        r#"unit App.Shared;
public function Base(): integer;
begin
  return 100;
end function;
end unit;

"#,
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "211\n");
}

#[test]
fn three_unit_cyclic_dependency() {
    let cwd = create_temp_dir("run-three-cycle");
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
uses App.C as C;
end unit;

"#,
    );
    write_text(
        &cwd.join("src/c.fpas"),
        r#"unit App.C;
uses App.A as A;
end unit;

"#,
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("Cyclic unit dependency detected"),
        "expected cycle error, got: {stderr_output}"
    );
}

#[test]
fn self_import_reports_cycle() {
    let cwd = create_temp_dir("run-self-import");
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
uses App.A as A;
end unit;

"#,
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("Cyclic unit dependency detected"),
        "expected cycle error, got: {stderr_output}"
    );
}

#[test]
fn library_dependency_diamond_does_not_warn_about_shared_sources() {
    let cwd = create_temp_dir("library-diamond-warning");
    let workspace = cwd.join("suite.fpasworkspace");
    write_text(
        &workspace,
        r#"[workspace]
name = "diamond"
members = ["common/lib.fpasprj", "left/lib.fpasprj", "right/lib.fpasprj", "app/app.fpasprj"]
"#,
    );
    for (name, dependencies, body) in [
        (
            "common",
            "",
            "public function Value(): integer; begin return 1; end function;",
        ),
        (
            "left",
            "workspace = [\"common\"]",
            "uses Repro.common as Common; public function LeftValue(): integer; begin return Common.Value(); end function;",
        ),
        (
            "right",
            "workspace = [\"common\"]",
            "uses Repro.common as Common; public function RightValue(): integer; begin return Common.Value(); end function;",
        ),
    ] {
        write_text(
            &cwd.join(name).join("lib.fpasprj"),
            &format!(
                r#"[project]
name = "{name}"
kind = "library"
[sources]
include = ["src/*.fpas"]
[exports]
units = ["Repro.{name}"]
[dependencies]
{dependencies}
"#
            ),
        );
        write_text(
            &cwd.join(name).join("src/unit.fpas"),
            &format!("unit Repro.{name}; {body} end unit;"),
        );
    }
    let project = cwd.join("app/app.fpasprj");
    write_text(
        &project,
        r#"[project]
name = "app"
kind = "program"
main = "main.fpas"
[sources]
include = ["main.fpas"]
[dependencies]
workspace = ["left", "right"]
"#,
    );
    write_text(
        &cwd.join("app/main.fpas"),
        r#"program App;  uses Repro.left as left; uses Repro.right as right; uses Std.Console as Console; begin Console.WriteLn(left.LeftValue() + right.RightValue()); end program;"#,
    );
    for (command, path) in [
        ("check", &workspace),
        ("check", &project),
        ("run", &project),
    ] {
        let (code, stdout, stderr) = support::run_cli_args_and_capture_output(
            &[command.to_string(), path.to_string_lossy().into_owned()],
            &cwd,
        );
        assert_eq!(code, 0, "{command}: {stderr}");
        assert!(
            !stderr.contains("Duplicate source file"),
            "{command}: {stderr}"
        );
        if command == "run" {
            assert_eq!(stdout, "2\n");
        }
    }
    fs::remove_dir_all(&cwd).expect("remove fixture");
}
