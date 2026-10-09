//! Root-program import errors retain the current editor source and the unit-name range.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md`

use super::project_errors::{analyze_project, located_diagnostic, program_project};
use super::*;

#[test]
fn missing_root_unit_publishes_its_original_code_and_exclusive_name_range() {
    let temp = TempDirectory::new("missing-root-unit");
    let source =
        "program Demo; // 😀\r\nuses Std.Console, Demo.Missing as Missing;\r\nbegin\r\nend.\r\n";
    program_project(&temp, source);
    let transcript = analyze_project(&temp, source);
    assert_success(&transcript);
    let diagnostic = located_diagnostic(&transcript, &temp.uri("src/main.fpas"), "FP4115");
    assert_eq!(
        diagnostic["range"],
        json!({"start":{"line":1,"character":18},"end":{"line":1,"character":30}})
    );
    assert_eq!(diagnostic["severity"], json!(1));
    assert_eq!(diagnostic["data"]["source_id"], json!(0));
    super::project_errors::assert_diagnostic_source(diagnostic, &temp.path().join("src/main.fpas"));
    assert!(diagnostic["data"]["hint"].is_string());
    assert!(
        !notifications(&transcript.messages, "window/logMessage")
            .iter()
            .any(|message| message["params"]["message"]
                .as_str()
                .is_some_and(|text| text.contains("error[FP4115]")))
    );
}

#[test]
fn private_root_unit_publishes_on_the_program_instead_of_the_library_source() {
    let temp = TempDirectory::new("private-root-unit");
    let source = "program Demo;\nuses Library.Hidden as Hidden;\nbegin\nend.\n";
    program_project(&temp, source);
    temp.write("demo.fpasprj", "[project]\nname = 'demo'\nkind = 'program'\nmain = 'src/main.fpas'\n[sources]\ninclude = ['src/**/*.fpas']\n[dependencies]\nprojects = ['library/library.fpasprj']\n");
    temp.write("library/library.fpasprj", "[project]\nname = 'library'\nkind = 'library'\n[sources]\ninclude = ['src/*.fpas']\n[exports]\nunits = ['Library.Api']\n");
    temp.write("library/src/api.fpas", "unit Library.Api; end unit;\n");
    temp.write(
        "library/src/hidden.fpas",
        "unit Library.Hidden; end unit;\n",
    );
    let transcript = analyze_project(&temp, source);
    assert_success(&transcript);
    let diagnostic = located_diagnostic(&transcript, &temp.uri("src/main.fpas"), "FP4116");
    assert_eq!(
        diagnostic["range"],
        json!({"start":{"line":1,"character":5},"end":{"line":1,"character":19}})
    );
    super::project_errors::assert_diagnostic_source(diagnostic, &temp.path().join("src/main.fpas"));
    assert!(
        diagnostic["data"]["hint"]
            .as_str()
            .is_some_and(|hint| hint.contains("[exports].units"))
    );
    assert!(
        notifications(&transcript.messages, "textDocument/publishDiagnostics")
            .iter()
            .all(|message| message["params"]["uri"] == temp.uri("src/main.fpas"))
    );
}

#[test]
fn root_import_locations_follow_editor_changes_and_clear_after_correction() {
    let temp = TempDirectory::new("root-import-editor-version");
    let valid = "program Demo;\nbegin\nend.\n";
    let invalid = "program Demo;\nuses Demo.Missing as Missing;\nbegin\nend.\n";
    program_project(&temp, valid);
    let root = temp.uri(".");
    let main = temp.uri("src/main.fpas");
    let transcript = run_script(&[
        TranscriptStep::Message(initialize_with_root(1, Some(&root))),
        TranscriptStep::Message(initialized()),
        TranscriptStep::Message(open(&main, 1, valid)),
        TranscriptStep::Wait(ANALYSIS_WAIT),
        TranscriptStep::Message(change(&main, 2, invalid)),
        TranscriptStep::Wait(ANALYSIS_WAIT),
        TranscriptStep::Message(change(&main, 3, valid)),
        TranscriptStep::Wait(ANALYSIS_WAIT),
        TranscriptStep::Message(shutdown(2)),
        TranscriptStep::Message(exit()),
    ]);
    assert_success(&transcript);
    let messages = notifications(&transcript.messages, "textDocument/publishDiagnostics");
    let invalid = publication(&messages, 2);
    assert_eq!(invalid["params"]["uri"], main);
    assert_eq!(invalid["params"]["diagnostics"][0]["code"], "FP4115");
    assert_eq!(
        invalid["params"]["diagnostics"][0]["range"],
        json!({"start":{"line":1,"character":5},"end":{"line":1,"character":17}})
    );
    assert_eq!(
        publication(&messages, 3)["params"]["diagnostics"],
        json!([])
    );
}
