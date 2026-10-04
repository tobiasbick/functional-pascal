//! Editor signatures and parameter modifiers retain explicit caller-storage modes.

#![allow(
    clippy::expect_used,
    reason = "editor fixtures use exact source positions"
)]

mod support;

use fpas_language_service::{LanguageService, SemanticTokenKind, WorkspaceContext};
use support::TempDirectory;

#[test]
fn source_and_expression_target_signatures_retain_var_modes() {
    let temp = TempDirectory::new("caller-reference-signatures");
    let source = r#"program T; type Action = procedure(var Value: integer); procedure Change(var Target: integer); begin Target := Target + 1; end procedure; function Select(): Action; begin return Change; end function; begin  var Count: integer := 1; Change(var Count); Select()(var Count); end program;"#;
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    for (call, formal) in [
        ("Change(var Count)", "var Target: integer"),
        ("Select()(var Count)", "var Value: integer"),
    ] {
        let offset =
            source.find(call).expect("call") + call.find("var Count").expect("argument") + 4;
        let help = service
            .signature_help(&path, offset)
            .expect("signature")
            .value
            .expect("typed call");
        assert_eq!(help.signature.parameters, [formal]);
    }
}

#[test]
fn semantic_tokens_distinguish_var_parameters_from_value_snapshots() {
    let temp = TempDirectory::new("caller-reference-tokens");
    let source = r#"program T; procedure Change(var Target: integer; Snapshot: integer); begin Target := Snapshot; end procedure; begin  var Count: integer := 1; Change(var Count, Count); end program;"#;
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let tokens = service.semantic_tokens(&path).expect("tokens").value;
    for (name, readonly) in [("Target", false), ("Snapshot", true)] {
        let token = tokens
            .iter()
            .find(|token| {
                token.kind == SemanticTokenKind::Parameter
                    && &source[token.span.offset()..token.span.end()] == name
            })
            .expect("parameter token");
        assert_eq!(token.modifiers.readonly, readonly);
    }
}

#[test]
fn field_completion_excludes_ordinary_value_and_var_callables() {
    let temp = TempDirectory::new("caller-reference-completion");
    let source = r#"program T; procedure Change(var Target: integer); begin Target := Target + 1; end procedure; function Copy(Value: integer): integer; begin return Value; end function; begin  var Count: integer := 1; discard Count.Copy(); end program;"#;
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let cursor = source.find("Count.Copy").expect("receiver call") + "Count.".len();
    let candidates = service
        .completions(&path, cursor)
        .expect("completion")
        .value;
    assert!(
        !candidates.iter().any(|candidate| candidate.label == "Copy"),
        "{candidates:#?}"
    );
    assert!(
        !candidates
            .iter()
            .any(|candidate| candidate.label == "Change"),
        "{candidates:#?}"
    );
}
