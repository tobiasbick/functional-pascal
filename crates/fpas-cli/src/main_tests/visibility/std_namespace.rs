use super::*;

#[test]
fn unknown_std_unit_rejected_with_available_units_hint() {
    let (exit_code, _, stderr_output) = support::run_source_and_capture_output(
        "unknown_std.fpas",
        "program Test;\nuses Std.Nonexistent as Nonexistent;\nbegin\nend.\n",
    );

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("Unknown standard library unit"),
        "expected unknown std unit error, got: {stderr_output}"
    );
    // The error message should hint at valid units
    assert!(
        stderr_output.contains("Std.Console") || stderr_output.contains("Available"),
        "expected available units hint, got: {stderr_output}"
    );
}

#[test]
fn bare_std_uses_rejected() {
    let (exit_code, _, stderr_output) = support::run_source_and_capture_output(
        "bare_std.fpas",
        "program Test;\nuses Std as StdUnit;\nbegin\nend.\n",
    );

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("Std"),
        "expected Std namespace error, got: {stderr_output}"
    );
}

#[test]
fn std_uses_case_insensitive() {
    let (exit_code, stdout_output, stderr_output) = support::run_source_and_capture_output(
        "std_case.fpas",
        "program Test;\nuses std.console as console, sTd.TeXt.FoRmAt as Text, sTd.MaTh as Numbers;\nbegin\n  console.WriteLn(Text.RealToStr(Numbers.IntToReal(42)));\nend.\n",
    );

    assert_eq!(exit_code, 0, "stderr: {stderr_output}");
    assert_eq!(stdout_output, "42.0\n");
}

#[test]
fn retired_standard_imports_have_no_compatibility_aliases() {
    let cwd = create_temp_dir("retired-standard-imports");
    let program = cwd.join("main.fpas");
    let library = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../lib");
    for unit in [
        "Std.Acp",
        "Std.Acp.Types",
        "Std.Acp.Host",
        "Std.Acp.Experimental",
        "Std.Acp.V2",
        "Std.Mcp",
        "Std.Mcp.Types",
        "Std.Net",
        "Std.Http",
        "Std.Server",
        "Std.Fs",
        "Std.Path",
        "Std.Args",
        "Std.Env",
        "Std.Proc",
        "Std.Errors",
        "Std.Tui",
        "Std.Conv",
        "Std.Parse",
        "Std.Net.Utf8",
        "Std.Json",
        "Std.Json.Fields",
        "Std.Toml",
        "Std.Toml.Fields",
        "Std.Bits",
        "Std.Random",
        "Std.Calendar",
    ] {
        let source = format!("program Removed;\nuses {unit} as Old;\nbegin\nend.\n");
        write_text(&program, &source);
        let (exit, _, stderr) = support::run_cli_args_and_capture_output(
            &[
                "check".into(),
                "--std-lib".into(),
                library.to_string_lossy().into_owned(),
                program.to_string_lossy().into_owned(),
            ],
            &cwd,
        );
        assert_eq!(exit, 1, "retired import {unit} was accepted: {stderr}");
        assert!(stderr.contains("FP4115"), "{unit}: {stderr}");
    }
    fs::remove_dir_all(&cwd).expect("fixture cleanup");
}

#[test]
fn namespace_parents_do_not_import_children() {
    for unit in ["Std.Ai", "Std.Io", "Std.Os", "Std.Text", "Std.Ui"] {
        let source = format!("program Parent;\nuses {unit} as ParentUnit;\nbegin\nend.\n");
        let (exit, _, stderr) = support::run_source_and_capture_output("std_parent.fpas", &source);
        assert_eq!(exit, 1, "namespace parent {unit} was accepted: {stderr}");
        assert!(
            stderr.contains("Unknown standard library unit"),
            "{unit}: {stderr}"
        );
    }
}

#[test]
fn relocated_source_children_remain_private() {
    let cwd = create_temp_dir("private-relocated-standard-imports");
    let program = cwd.join("main.fpas");
    let library = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../lib");
    for unit in [
        "Std.Ui.Terminal.Runtime.Application",
        "Std.Net.Http.Wire",
        "Std.Time.Calendar.Text",
    ] {
        write_text(
            &program,
            &format!("program Hidden;\nuses {unit} as HiddenUnit;\nbegin\nend.\n"),
        );
        let (exit, _, stderr) = support::run_cli_args_and_capture_output(
            &[
                "check".into(),
                "--std-lib".into(),
                library.to_string_lossy().into_owned(),
                program.to_string_lossy().into_owned(),
            ],
            &cwd,
        );
        assert_eq!(exit, 1, "private import {unit} was accepted: {stderr}");
        assert!(stderr.contains("FP4116"), "{unit}: {stderr}");
    }
    fs::remove_dir_all(&cwd).expect("fixture cleanup");
}

#[test]
fn std_unit_with_extra_segments_rejected() {
    let (exit_code, _, stderr_output) = support::run_source_and_capture_output(
        "std_extra_seg.fpas",
        "program Test;\nuses Std.Console.Extra as Extra;\nbegin\nend.\n",
    );

    assert_eq!(exit_code, 1);
    assert!(
        stderr_output.contains("Std"),
        "expected Std segment error, got: {stderr_output}"
    );
}

#[test]
fn user_unit_cannot_use_std_namespace_even_when_unused() {
    let cwd = create_temp_dir("run-user-std-ns");
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
    write_text(&cwd.join("src/main.fpas"), "program Main;\nbegin\nend.\n");
    write_text(
        &cwd.join("src/mylib.fpas"),
        "unit sTd.MyLib;\nfunction Foo(): integer;\nbegin\n  return 1;\nend function;\nend unit;\n",
    );

    let (exit_code, _, stderr_output) = support::run_cli_and_capture_output(&project_file, &cwd);
    fs::remove_dir_all(&cwd).expect("temp directory must be removed");

    assert_eq!(exit_code, 1, "stderr: {stderr_output}");
    assert!(
        stderr_output.contains("reserved for standard library units"),
        "expected reserved namespace error, got: {stderr_output}"
    );
}
