//! Existing source, dependency and standard-library diagnostic workflows.
//!
//! **Documentation:** `docs/pascal/tools/editor-integration.md`

use super::*;

#[test]
fn project_dependency_diagnostic_uses_the_dependency_uri_and_range() {
    let temp = TempDirectory::new("dependency-diagnostic");
    temp.write(
        "demo.fpasprj",
        r#"[project]
name = "demo"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]
"#,
    );
    temp.write(
        "src/main.fpas",
        "program App;\n\nuses Demo.Math;\n\nbegin\n  const Value: integer := Answer();\nend.\n",
    );
    let unit_source = "unit Demo.Math;\n\npublic function Answer(): integer;\nbegin\n  return 'wrong';\nend function;\nend unit;\n";
    temp.write("src/math.fpas", unit_source);
    let root_uri = temp.uri(".");
    let unit_uri = temp.uri("src/math.fpas");
    let transcript = run_script(&[
        TranscriptStep::Message(initialize_with_root(1, Some(&root_uri))),
        TranscriptStep::Message(initialized()),
        TranscriptStep::Message(open(&unit_uri, 1, unit_source)),
        TranscriptStep::Wait(ANALYSIS_WAIT),
        TranscriptStep::Message(shutdown(2)),
        TranscriptStep::Message(exit()),
    ]);

    assert_success(&transcript);
    let published = notifications(&transcript.messages, "textDocument/publishDiagnostics");
    let publication = publication(&published, 1);
    assert_eq!(publication["params"]["uri"], json!(unit_uri));
    let diagnostics = publication["params"]["diagnostics"]
        .as_array()
        .expect("dependency diagnostics");
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic["code"]
                .as_str()
                .is_some_and(|code| code.starts_with("FP3"))
                && diagnostic["range"]["start"]["line"] == json!(4)
        }),
        "{diagnostics:?}"
    );
}

#[test]
fn unreadable_sibling_publishes_current_syntax_and_logs_coded_read_failure() {
    for (invalid_utf8, code) in [(false, "FP4101"), (true, "FP4102")] {
        let temp = TempDirectory::new("missing-sibling-diagnostics");
        temp.write(
            "demo.fpasprj",
            r#"[project]
name = "demo"
kind = "program"
main = "src/main.fpas"

[sources]
include = ["src/**/*.fpas"]
"#,
        );
        let valid =
            "program App;\n\nuses Demo.Math;\n\nbegin\n  const Value: integer := Answer();\nend.\n";
        let unit_source = "unit Demo.Math;\n\npublic function Answer(): integer;\nbegin\n  return 42;\nend function;\nend unit;\n";
        let main = temp.write("src/main.fpas", valid);
        let unit = temp.write("src/math.fpas", unit_source);
        let root_uri = temp.uri(".");
        let main_uri = temp.uri("src/main.fpas");
        let remove_unit = unit.clone();
        let restore_unit = unit.clone();
        let transcript = run_script(&[
            TranscriptStep::Message(initialize_with_root(1, Some(&root_uri))),
            TranscriptStep::Message(initialized()),
            TranscriptStep::Message(open(&main_uri, 1, valid)),
            TranscriptStep::Wait(ANALYSIS_WAIT),
            TranscriptStep::Action(Box::new(move || {
                if invalid_utf8 {
                    std::fs::write(&remove_unit, [0xff]).expect("invalid UTF-8 sibling");
                } else {
                    std::fs::remove_file(&remove_unit).expect("remove sibling");
                }
            })),
            TranscriptStep::Message(change(
                &main_uri,
                2,
                "program Broken;\nbegin\n  if then; end if;\nend.\n",
            )),
            TranscriptStep::Wait(ANALYSIS_WAIT),
            TranscriptStep::Action(Box::new(move || {
                std::fs::write(&restore_unit, unit_source).expect("restore sibling")
            })),
            TranscriptStep::Message(change(&main_uri, 3, valid)),
            TranscriptStep::Wait(ANALYSIS_WAIT),
            TranscriptStep::Message(shutdown(2)),
            TranscriptStep::Message(exit()),
        ]);

        assert_success(&transcript);
        let published = notifications(&transcript.messages, "textDocument/publishDiagnostics");
        let version_2 = publication(&published, 2)["params"]["diagnostics"]
            .as_array()
            .expect("version two diagnostics");
        let logged = notifications(&transcript.messages, "window/logMessage");
        assert!(
            logged.iter().any(|message| message["params"]["message"]
                .as_str()
                .is_some_and(|message| message.contains(&format!("error[{code}]"))
                    && message.contains("math.fpas"))),
            "{logged:?}"
        );
        let hint = if invalid_utf8 {
            "Save the source file as UTF-8."
        } else {
            "Check that the source file exists and is readable."
        };
        assert!(
            logged.iter().any(|message| message["params"]["message"]
                .as_str()
                .is_some_and(|message| message.contains(hint))),
            "{logged:?}"
        );
        assert!(
            version_2.iter().any(|diagnostic| diagnostic["code"]
                .as_str()
                .is_some_and(|code| code.starts_with("FP2"))),
            "{version_2:?}"
        );
        assert_eq!(
            publication(&published, 3)["params"]["diagnostics"],
            json!([])
        );
        assert!(main.exists());
    }
}

#[test]
fn configured_standard_library_does_not_duplicate_a_nested_standard_library() {
    let temp = TempDirectory::new("nested-standard-library");
    temp.write(
        "bundle/stdlib.fpasprj",
        r#"[project]
name = "bundled-stdlib"
kind = "library"

[sources]
include = ["Std/**/*.fpas"]
"#,
    );
    temp.write(
        "bundle/Std/Point.fpas",
        "unit Std.Point;\n\npublic type Point = integer;\nend unit;\n",
    );
    temp.write(
        "repository/lib/stdlib.fpasprj",
        r#"[project]
name = "test-stdlib"
kind = "library"

[sources]
include = ["Std/**/*.fpas"]
"#,
    );
    temp.write(
        "repository/lib/Std/Point.fpas",
        r#"unit Std.Point;

public type Point = record
  public X: integer;
end record;
end unit;
"#,
    );
    let facade_source = r#"unit Std.Facade;

uses Std.Point;

public type FacadePoint = Std.Point.Point;
end unit;
"#;
    temp.write("repository/lib/Std/Facade.fpas", facade_source);
    let root_uri = temp.uri("repository");
    let standard_library_uri = temp.uri("bundle");
    let facade_uri = temp.uri("repository/lib/Std/Facade.fpas");
    let transcript = run_script(&[
        TranscriptStep::Message(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "processId": null,
                "rootUri": root_uri,
                "capabilities": {},
                "initializationOptions": {
                    "standardLibraryUri": standard_library_uri
                }
            }
        })),
        TranscriptStep::Message(initialized()),
        TranscriptStep::Message(open(&facade_uri, 1, facade_source)),
        TranscriptStep::Wait(ANALYSIS_WAIT),
        TranscriptStep::Message(shutdown(2)),
        TranscriptStep::Message(exit()),
    ]);

    assert_success(&transcript);
    let published = notifications(&transcript.messages, "textDocument/publishDiagnostics");
    assert_eq!(
        publication(&published, 1)["params"]["diagnostics"],
        json!([])
    );
}

#[test]
fn initialized_standard_library_resolves_std_units_for_an_external_project() {
    let temp = TempDirectory::new("configured-standard-library");
    temp.write(
        "bundle/stdlib.fpasprj",
        r#"[project]
name = "test-stdlib"
kind = "library"

[exports]
units = ["Std.Tui"]

[sources]
include = ["Std/**/*.fpas"]
"#,
    );
    temp.write(
        "bundle/Std/Tui.fpas",
        "unit Std.Tui;\n\npublic type TuiPalette = integer;\nend unit;\n",
    );
    temp.write(
        "external/external.fpasprj",
        r#"[project]
name = "external"
kind = "program"
main = "main.fpas"

[sources]
include = ["main.fpas"]
"#,
    );
    let source =
        "program External;\n\nuses Std.Tui;\n\nbegin\n  const Palette: TuiPalette := 1;\nend.\n";
    temp.write("external/main.fpas", source);
    let root_uri = temp.uri("external");
    let standard_library_uri = temp.uri("bundle");
    let source_uri = temp.uri("external/main.fpas");
    let transcript = run_script(&[
        TranscriptStep::Message(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "processId": null,
                "rootUri": root_uri,
                "capabilities": {},
                "initializationOptions": {
                    "standardLibraryUri": standard_library_uri
                }
            }
        })),
        TranscriptStep::Message(initialized()),
        TranscriptStep::Message(open(&source_uri, 1, source)),
        TranscriptStep::Wait(ANALYSIS_WAIT),
        TranscriptStep::Message(shutdown(2)),
        TranscriptStep::Message(exit()),
    ]);

    assert_success(&transcript);
    let published = notifications(&transcript.messages, "textDocument/publishDiagnostics");
    assert_eq!(
        publication(&published, 1)["params"]["diagnostics"],
        json!([])
    );
}
