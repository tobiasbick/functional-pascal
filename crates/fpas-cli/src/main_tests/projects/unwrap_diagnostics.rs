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
        ("option of TuiStyle", "Some"),
        ("result of (TuiStyle, string)", "Ok"),
    ] {
        for function in ["Unwrap", "UnwrapOr"] {
            for namespace in [None, Some("Options"), Some("Results")] {
                let fallback = if function == "UnwrapOr" { "Base" } else { "" };
                let call = match namespace {
                    None => format!("Style.{function}({fallback})"),
                    Some(namespace) => {
                        let tail = if fallback.is_empty() { "" } else { ", Base" };
                        format!("Std.{namespace}.{function}(Style{tail})")
                    }
                };
                write_text(
                    &cwd.join("src/main.fpas"),
                    &format!(
                        "program UnwrapRepro;\nuses Std.Tui, Std.Test;\nbegin\n  const Base: TuiStyle := TuiStyle.FromColors(\n    TuiColor.FromRgb(1, 2, 3), TuiColor.FromRgb(4, 5, 6));\n  const Style: {container} := {constructor}(Base);\n  const Red: integer := {call}.Background.Red;\n  AssertEquals(4, Red);\nend."
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
    }
    fs::remove_dir_all(&cwd).expect("remove fixture");
}
