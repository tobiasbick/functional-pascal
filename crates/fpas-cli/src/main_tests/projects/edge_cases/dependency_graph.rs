use super::*;

#[test]
fn diamond_dependency_graph() {
    let cwd = create_temp_dir("run-diamond-deps");
    let project_file = cwd.join("app.fpasprj");
    support::write_program_project_file(&project_file, "src/main.fpas", &["src/*.fpas"]);
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses App.A, App.B, Std.Console;\nbegin\n  WriteLn(FromA() + FromB())\nend.\n",
    );
    write_text(
        &cwd.join("src/a.fpas"),
        "unit App.A;\nuses App.Shared;\npublic function FromA(): integer;\nbegin\n  return Base() + 1\nend;\n",
    );
    write_text(
        &cwd.join("src/b.fpas"),
        "unit App.B;\nuses App.Shared;\npublic function FromB(): integer;\nbegin\n  return Base() + 10\nend;\n",
    );
    write_text(
        &cwd.join("src/shared.fpas"),
        "unit App.Shared;\npublic function Base(): integer;\nbegin\n  return 100\nend;\n",
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
        "program Main;\nuses App.A;\nbegin\nend.\n",
    );
    write_text(&cwd.join("src/a.fpas"), "unit App.A;\nuses App.B;\n");
    write_text(&cwd.join("src/b.fpas"), "unit App.B;\nuses App.C;\n");
    write_text(&cwd.join("src/c.fpas"), "unit App.C;\nuses App.A;\n");

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
        "program Main;\nuses App.A;\nbegin\nend.\n",
    );
    write_text(&cwd.join("src/a.fpas"), "unit App.A;\nuses App.A;\n");

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
            "public function Value(): integer; begin return 1 end;",
        ),
        (
            "left",
            "workspace = [\"common\"]",
            "uses Repro.common; public function LeftValue(): integer; begin return Value() end;",
        ),
        (
            "right",
            "workspace = [\"common\"]",
            "uses Repro.common; public function RightValue(): integer; begin return Value() end;",
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
            &format!("unit Repro.{name}; {body}"),
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
        "program App; uses Repro.left, Repro.right, Std.Console; begin WriteLn(LeftValue() + RightValue()) end.",
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
