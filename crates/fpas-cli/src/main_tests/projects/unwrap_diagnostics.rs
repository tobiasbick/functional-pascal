//! Type diagnostics for unwrapping the wrong container before field access.

use super::*;

#[test]
fn unwrap_namespace_mismatch_reports_a_type_error_before_lowering() {
    let cwd = create_temp_dir("unwrap-namespace");
    let project = cwd.join("repro.fpasprj");
    support::write_program_project_file(&project, "src/main.fpas", &["src/**/*.fpas"]);
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("repository root");
    for (container, constructor, correct, wrong) in [
        (
            "option of (Tui.TuiStyle)",
            "Option.Some",
            "Options",
            "Results",
        ),
        (
            "result of (Tui.TuiStyle, string)",
            "Result.Ok",
            "Results",
            "Options",
        ),
    ] {
        for function in ["Unwrap", "UnwrapOr"] {
            for namespace in [wrong, correct] {
                let fallback = if function == "UnwrapOr" { ", Base" } else { "" };
                write_text(
                    &cwd.join("src/main.fpas"),
                    &format!(
                        r#"program UnwrapRepro;
uses Std.Options as Options; uses Std.Results as Results; uses Std.Tui as Tui; uses Std.Test as Test;
uses Std.Tui.Cells as Cells;
begin
  const Base: Tui.TuiStyle := Cells.TuiStyleFromColors(
    Cells.TuiColorFromRgb(1, 2, 3), Cells.TuiColorFromRgb(4, 5, 6));
  const Style: {container} := {constructor}(Base);
  const Red: integer := {namespace}.{function}(Style{fallback}).Background.Red;
  Test.AssertEquals(4, Red);
end program;"#
                    ),
                );
                let command = if namespace == correct { "run" } else { "check" };
                let (code, _, stderr) = support::run_cli_args_and_capture_output(
                    &[
                        command.to_string(),
                        "--std-lib".to_string(),
                        root.join("lib").to_string_lossy().into_owned(),
                        project.to_string_lossy().into_owned(),
                    ],
                    &cwd,
                );
                if namespace == correct {
                    assert_eq!(code, 0, "{namespace}.{function}: {stderr}");
                } else {
                    assert_ne!(code, 0, "{namespace}.{function}");
                    assert!(stderr.contains("F2006"), "{stderr}");
                    assert!(
                        stderr.contains(&format!("Std.{namespace}.{function}")),
                        "{stderr}"
                    );
                    assert!(!stderr.contains("F9001"), "{stderr}");
                }
            }
        }
    }
    fs::remove_dir_all(&cwd).expect("remove fixture");
}
