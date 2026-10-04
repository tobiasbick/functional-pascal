//! Checked local inference drives editor details, fields and callable signatures.

#![allow(
    clippy::expect_used,
    reason = "editor tests use exact source positions"
)]
mod support;
use fpas_language_service::{LanguageService, WorkspaceContext};
use support::TempDirectory;

#[test]
fn inferred_record_fields_and_callable_modes_reach_editor_queries() {
    let temp = TempDirectory::new("binding-inference");
    let source = "program T; type Point = record X: integer; end record;
        procedure Change(var Target: integer); begin Target := Target + 1; end procedure;
        begin var Position := Point(X := 1); const Action := Change;
          Action(var Position.X); discard Position.X; end program;";
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let hover = service
        .hover(&path, source.rfind("Position.X").expect("read"))
        .expect("hover")
        .value
        .expect("binding");
    assert_eq!(hover.contents, "var Position: Point");
    let completion = service
        .completions(
            &path,
            source.rfind("Position.X").expect("field") + "Position.".len(),
        )
        .expect("completion")
        .value;
    assert!(
        completion.iter().any(|candidate| candidate.label == "X"),
        "{completion:#?}"
    );
    let help = service
        .signature_help(
            &path,
            source.find("Action(var Position").expect("call") + "Action(var ".len(),
        )
        .expect("signature")
        .value
        .expect("inferred callable");
    assert_eq!(help.signature.parameters, ["var Target: integer"]);
}
