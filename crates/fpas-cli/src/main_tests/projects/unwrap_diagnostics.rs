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
        ("option of TuiStyle", "Some", "Options", "Results"),
        ("result of TuiStyle, string", "Ok", "Results", "Options"),
    ] {
        for function in ["Unwrap", "UnwrapOr"] {
            for namespace in [wrong, correct] {
                let fallback = if function == "UnwrapOr" { ", Base" } else { "" };
                write_text(
                    &cwd.join("src/main.fpas"),
                    &format!(
                        "program UnwrapRepro;
uses Std.Options, Std.Results, Std.Tui, Std.Test;
begin
  var Base: TuiStyle := TuiStyle.FromColors(
    TuiColor.FromRgb(1, 2, 3), TuiColor.FromRgb(4, 5, 6));
  var Style: {container} := {constructor}(Base);
  var Red: integer := Std.{namespace}.{function}(Style{fallback}).Background.Red;
  AssertEquals(4, Red)
end."
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
