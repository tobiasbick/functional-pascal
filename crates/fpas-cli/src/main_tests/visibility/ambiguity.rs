use super::*;

#[test]
fn unqualified_name_lists_three_required_aliases() {
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
        "program Main;\nuses App.A as A, App.B as B, App.C as C;\nbegin\n  Compute();\nend.\n",
    );
    write_text(
        &cwd.join("src/a.fpas"),
        "unit App.A;\npublic function Compute(): integer;\nbegin\n  return 1;\nend function;\nend unit;\n",
    );
    write_text(
        &cwd.join("src/b.fpas"),
        "unit App.B;\npublic function Compute(): integer;\nbegin\n  return 2;\nend function;\nend unit;\n",
    );
    write_text(
        &cwd.join("src/c.fpas"),
        "unit App.C;\npublic function Compute(): integer;\nbegin\n  return 3;\nend function;\nend unit;\n",
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("Name `Compute` requires an explicit qualifier"),
        "expected qualification error, got: {stderr_output}"
    );
}

#[test]
fn qualification_error_mentions_alias_qualified_alternatives() {
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
        "program Main;\nuses App.X as X, App.Y as Y;\nbegin\n  Foo();\nend.\n",
    );
    write_text(
        &cwd.join("src/x.fpas"),
        "unit App.X;\npublic function Foo(): integer;\nbegin\n  return 1;\nend function;\nend unit;\n",
    );
    write_text(
        &cwd.join("src/y.fpas"),
        "unit App.Y;\npublic function Foo(): integer;\nbegin\n  return 2;\nend function;\nend unit;\n",
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("X.Foo") && stderr_output.contains("Y.Foo"),
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
        "program Main;\nuses App.A as A, App.B as B, Std.Console as Console;\nbegin\n  Console.WriteLn(B.Compute());\nend.\n",
    );
    write_text(
        &cwd.join("src/a.fpas"),
        "unit App.A;\n\nfunction Compute(): integer;\nbegin\n  return 1;\nend function;\nend unit;\n",
    );
    write_text(
        &cwd.join("src/b.fpas"),
        "unit App.B;\n\npublic function Compute(): integer;\nbegin\n  return 2;\nend function;\nend unit;\n",
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
        "program Main;\nuses App.Math as Math, App.Advanced as Advanced, Std.Console as Console;\nbegin\n  Console.WriteLn(Math.Add(1, 2));\n  Console.WriteLn(Advanced.Add(10, 20));\nend.\n",
    );
    write_text(
        &cwd.join("src/math.fpas"),
        "unit App.Math;\npublic function Add(A: integer; B: integer): integer;\nbegin\n  return A + B;\nend function;\nend unit;\n",
    );
    write_text(
        &cwd.join("src/advanced.fpas"),
        "unit App.Advanced;\npublic function Add(A: integer; B: integer): integer;\nbegin\n  return A * B;\nend function;\nend unit;\n",
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
        "program Main;\nuses App.Math as Math, App.Advanced as Advanced;\nbegin\nend.\n",
    );
    write_text(
        &cwd.join("src/math.fpas"),
        "unit App.Math;\npublic function Add(A: integer; B: integer): integer;\nbegin\n  return A + B;\nend function;\nend unit;\n",
    );
    write_text(
        &cwd.join("src/advanced.fpas"),
        "unit App.Advanced;\npublic function Add(A: integer; B: integer): integer;\nbegin\n  return A * B;\nend function;\nend unit;\n",
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    // No error because the ambiguous short name `Add` is never referenced
    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
}
