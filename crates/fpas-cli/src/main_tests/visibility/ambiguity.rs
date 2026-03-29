use super::*;

#[test]
fn ambiguity_with_three_units() {
    let cwd = create_temp_dir("vis-ambig-three");
    let project_file = cwd.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/*.fpas"]
"#,
    );
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses App.A, App.B, App.C;\nbegin\n  Compute()\nend.\n",
    );
    write_text(
        &cwd.join("src/a.fpas"),
        "unit App.A;\nfunction Compute(): integer;\nbegin\n  return 1\nend;\n",
    );
    write_text(
        &cwd.join("src/b.fpas"),
        "unit App.B;\nfunction Compute(): integer;\nbegin\n  return 2\nend;\n",
    );
    write_text(
        &cwd.join("src/c.fpas"),
        "unit App.C;\nfunction Compute(): integer;\nbegin\n  return 3\nend;\n",
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("Ambiguous imported symbol `Compute`"),
        "expected ambiguity error, got: {stderr_output}"
    );
}

#[test]
fn ambiguity_error_mentions_qualified_alternatives() {
    let cwd = create_temp_dir("vis-ambig-hint");
    let project_file = cwd.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/*.fpas"]
"#,
    );
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses App.X, App.Y;\nbegin\n  Foo()\nend.\n",
    );
    write_text(
        &cwd.join("src/x.fpas"),
        "unit App.X;\nfunction Foo(): integer;\nbegin\n  return 1\nend;\n",
    );
    write_text(
        &cwd.join("src/y.fpas"),
        "unit App.Y;\nfunction Foo(): integer;\nbegin\n  return 2\nend;\n",
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("App.X.Foo") && stderr_output.contains("App.Y.Foo"),
        "error should mention qualified alternatives, got: {stderr_output}"
    );
}

#[test]
fn private_does_not_cause_ambiguity() {
    let cwd = create_temp_dir("vis-no-ambiguity");
    let project_file = cwd.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/*.fpas"]
"#,
    );
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses App.A, App.B, Std.Console;\nbegin\n  WriteLn(Compute())\nend.\n",
    );
    write_text(
        &cwd.join("src/a.fpas"),
        "unit App.A;\n\nprivate function Compute(): integer;\nbegin\n  return 1\nend;\n",
    );
    write_text(
        &cwd.join("src/b.fpas"),
        "unit App.B;\n\nfunction Compute(): integer;\nbegin\n  return 2\nend;\n",
    );

    let (exit_code, stdout_output, _) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0);
    assert_eq!(stdout_output, "2\n");
}

#[test]
fn ambiguity_resolved_by_qualified_name() {
    let cwd = create_temp_dir("vis-ambig-qualified");
    let project_file = cwd.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/*.fpas"]
"#,
    );
    // Two units export the same short name — use qualified names to disambiguate
    write_text(
        &cwd.join("src/main.fpas"),
        "\
program Main;
uses App.Math, App.Advanced, Std.Console;
begin
  WriteLn(App.Math.Add(1, 2));
  WriteLn(App.Advanced.Add(10, 20))
end.
",
    );
    write_text(
        &cwd.join("src/math.fpas"),
        "unit App.Math;\nfunction Add(A: integer; B: integer): integer;\nbegin\n  return A + B\nend;\n",
    );
    write_text(
        &cwd.join("src/advanced.fpas"),
        "unit App.Advanced;\nfunction Add(A: integer; B: integer): integer;\nbegin\n  return A * B\nend;\n",
    );

    let (exit_code, stdout_output, stderr_output) =
        support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "3\n200\n");
}

#[test]
fn no_error_at_uses_site_when_ambiguous_name_not_used() {
    let cwd = create_temp_dir("vis-unused-ambiguity");
    let project_file = cwd.join("app.fpasprj");
    write_text(
        &project_file,
        r#"[project]
name = "app"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/*.fpas"]
"#,
    );
    // Import two units with conflicting short names but never use the ambiguous name
    write_text(
        &cwd.join("src/main.fpas"),
        "program Main;\nuses App.Math, App.Advanced;\nbegin\nend.\n",
    );
    write_text(
        &cwd.join("src/math.fpas"),
        "unit App.Math;\nfunction Add(A: integer; B: integer): integer;\nbegin\n  return A + B\nend;\n",
    );
    write_text(
        &cwd.join("src/advanced.fpas"),
        "unit App.Advanced;\nfunction Add(A: integer; B: integer): integer;\nbegin\n  return A * B\nend;\n",
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    // No error because the ambiguous short name `Add` is never referenced
    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
}
