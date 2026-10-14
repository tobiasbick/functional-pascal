//! Native unwrap chains and migration diagnostics before field-access lowering.

use super::*;

#[test]
fn native_unwrap_chains_and_retired_namespace_diagnostics() {
    let cwd = create_temp_dir("unwrap-namespace");
    let project = cwd.join("repro.fpasprj");
    support::write_program_project_file(&project, "src/main.fpas", &["src/**/*.fpas"]);
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("repository root");
    for (container, constructor) in [
        ("option of Tui.TuiStyle", "Some"),
        ("result of (Tui.TuiStyle, string)", "Ok"),
    ] {
        let function = "Unwrap";
        for namespace in [None, Some("Options"), Some("Results")] {
            let call = match namespace {
                None => "Style.Unwrap()".to_owned(),
                Some(namespace) => format!("Std.{namespace}.Unwrap(Style)"),
            };
            write_text(
                &cwd.join("src/main.fpas"),
                &format!(
                    "program UnwrapRepro;\nuses Std.Tui as Tui, Std.Test as Test;\nbegin\n  const Base: Tui.TuiStyle := Tui.TuiStyle.FromColors(\n    Tui.TuiColor.FromRgb(1, 2, 3), Tui.TuiColor.FromRgb(4, 5, 6));\n  const Style: {container} := {constructor}(Base);\n  const Red: integer := {call}.Background.Red;\n  Test.AssertEquals(4, Red);\nend."
                ),
            );
            let command = if namespace.is_none() { "run" } else { "check" };
            let (code, _, stderr) = support::run_cli_args_and_capture_output(
                &[
                    command.to_string(),
                    "--std-lib".to_string(),
                    root.join("lib").to_string_lossy().into_owned(),
                    project.to_string_lossy().into_owned(),
                ],
                &cwd,
            );
            if namespace.is_none() {
                assert_eq!(code, 0, "{call}: {stderr}");
            } else {
                assert_ne!(code, 0, "{call}");
                assert!(stderr.contains("FP3003"), "{stderr}");
                assert!(
                    stderr.contains(&format!("Use `Value.{function}(…)`")),
                    "{stderr}"
                );
                assert!(!stderr.contains("FP9001"), "{stderr}");
            }
        }
    }
    fs::remove_dir_all(&cwd).expect("remove fixture");
}
