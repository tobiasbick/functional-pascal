//! Closed-enum diagnostics through the CLI and imported type changes.
//!
//! **Documentation:** `docs/pascal/language/pattern-matching/exhaustiveness.md`

use super::{create_temp_dir, fs, support, write_text};
use crate::test_support::{write_library_fpasprj, write_program_fpasprj_with_deps};

#[test]
fn check_and_run_reject_enum_option_and_result_else_branches() {
    let dir = create_temp_dir("closed-enum-else");
    let path = dir.join("main.fpas");
    for (source, missing) in [
        (
            "program T; type Color = enum Red; Green; end enum;
             begin case Color.Red of when Color.Red: null; else null; end case; end.",
            "Color.Green",
        ),
        (
            "program T; begin case Some(1) of when Some(_): null; else null; end case; end.",
            "None",
        ),
        (
            "program T; const R: result of integer, string := Ok(1);
             begin case R of when Ok(_): null; else null; end case; end.",
            "Error(_)",
        ),
    ] {
        write_text(&path, source);
        for command in ["check", "run"] {
            let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(
                &[command.into(), path.to_string_lossy().into_owned()],
                &dir,
            );
            assert_eq!(exit, 1, "{command}: {stderr}");
            assert!(stdout.is_empty(), "{stdout}");
            assert!(stderr.contains("error[FP3035]"), "{stderr}");
            assert!(stderr.contains(missing), "{stderr}");
            assert!(stderr.contains("`null;`"), "{stderr}");
            assert!(!stderr.contains("FP3011"), "{stderr}");
        }
    }
    fs::remove_dir_all(dir).expect("remove closed-enum fixture");
}

#[test]
fn json_reports_a_located_redundant_enum_else_diagnostic() {
    let dir = create_temp_dir("closed-enum-else-json");
    let path = dir.join("main.fpas");
    write_text(
        &path,
        "program T;\nbegin\n  case Some(1) of when Some(_), None: null; else null; end case;\nend.\n",
    );
    let (exit, stdout, stderr) = support::run_cli_args_and_capture_output(
        &[
            "check".into(),
            "--diagnostics".into(),
            "json".into(),
            path.to_string_lossy().into_owned(),
        ],
        &dir,
    );
    fs::remove_dir_all(dir).expect("remove JSON fixture");
    assert_eq!(exit, 1);
    assert!(stdout.is_empty());
    let records = stderr
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).expect("diagnostic JSON"))
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 1, "{stderr}");
    let record = &records[0];
    assert_eq!(record["code"], "FP3035");
    assert_eq!(record["phase"], "sema");
    assert_eq!(record["location"]["start"]["line"], 3);
    assert_eq!(record["location"]["start"]["column"], 3);
    assert!(record["hint"].as_str().expect("hint").contains("redundant"));
}

#[test]
fn imported_enum_extension_invalidates_every_incomplete_consumer_case() {
    let dir = create_temp_dir("closed-enum-extension");
    let library = dir.join("Colors.fpas");
    write_library_fpasprj(&dir.join("library.fpasprj"), &["Colors.fpas"]);
    let project = dir.join("consumer.fpasprj");
    write_program_fpasprj_with_deps(
        &project,
        "consumer.fpas",
        &["consumer.fpas"],
        &["library.fpasprj"],
    );
    write_text(
        &library,
        "unit Colors; public type Color = enum Red; Green; end enum; end unit;",
    );
    let consumer = dir.join("consumer.fpas");
    let program = |extended: bool| {
        let extra = if extended {
            ", Imported.Color.Blue"
        } else {
            ""
        };
        format!(
            "program Consumer; uses Colors as Imported;
             const Value: Imported.Color := Imported.Color.Red;
             begin
               case Value of when Imported.Color.Red, Imported.Color.Green: null; end case;
               case Value of when Imported.Color.Red: null; when Imported.Color.Green: null; end case;
               case Value of when Imported.Color.Red, Imported.Color.Green{extra}: null; end case;
             end."
        )
    };
    write_text(&consumer, &program(false));
    for _ in 0..2 {
        let (exit, _, stderr) = support::run_cli_args_and_capture_output(
            &["run".into(), project.to_string_lossy().into_owned()],
            &dir,
        );
        assert_eq!(exit, 0, "{stderr}");
    }
    assert!(dir.join("Colors.fpascu").exists(), "missing unit sidecar");
    write_text(
        &library,
        "unit Colors; public type Color = enum Red; Green; Blue; end enum; end unit;",
    );
    write_text(&consumer, &program(true));
    let (exit, _, stderr) = support::run_cli_args_and_capture_output(
        &["check".into(), project.to_string_lossy().into_owned()],
        &dir,
    );
    assert_eq!(exit, 1, "{stderr}");
    assert_eq!(stderr.matches("error[FP3011]").count(), 2, "{stderr}");
    assert_eq!(
        stderr
            .to_ascii_lowercase()
            .matches("missing colors.color.blue")
            .count(),
        2,
        "{stderr}"
    );
    assert!(!stderr.contains("or an else"), "{stderr}");
    fs::remove_dir_all(dir).expect("remove enum-extension fixture");
}
