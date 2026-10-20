//! Canonical parser signatures and native wrapper completion in editor buffers.
//! See `docs/pascal/std/text/parse.md`.

use super::*;
use std::collections::BTreeSet;

#[test]
fn wrapper_completion_contains_only_retained_operations() {
    for (ty, value, family) in [
        ("Option of integer", "Some(1)", "Option"),
        ("Result of (integer, string)", "Ok(1)", "Result"),
    ] {
        let temp = TempDirectory::new("canonical-wrapper-completion");
        let source = format!("program T; begin const Value: {ty} := {value}; discard Value.\nend.");
        let path = temp.write("main.fpas", &source);
        let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
        let cursor = source.find("Value.\n").expect("receiver") + "Value.".len();
        let candidates = service
            .completions(&path, cursor)
            .expect("completion")
            .value;
        let actual = candidates
            .iter()
            .map(|candidate| candidate.qualified_name.clone())
            .collect::<BTreeSet<_>>();
        let mut names = vec!["Map", "AndThen", "OrElse", "Unwrap"];
        if family == "Result" {
            names.push("MapError");
        }
        let expected = names
            .into_iter()
            .map(|name| format!("{family}.{name}"))
            .collect();
        assert_eq!(actual, expected, "{candidates:#?}");
    }
}

#[test]
fn parser_alias_completion_and_signature_help_use_result_and_text() {
    let temp = TempDirectory::new("canonical-parser-completion");
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("repository");
    let source = "program T; uses Std.Parse as Numbers; begin discard Numbers.\nend.";
    let path = temp.write("main.fpas", source);
    let mut service = LanguageService::load_with_standard_library(temp.path(), &root.join("lib"))
        .expect("Std API");
    let cursor = source.find("Numbers.\n").expect("alias") + "Numbers.".len();
    let candidates = service
        .completions(&path, cursor)
        .expect("completion")
        .value;
    let actual = candidates
        .iter()
        .map(|candidate| candidate.label.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        actual,
        BTreeSet::from(["Int", "Real", "Bool", "HexInt", "ParseError"]),
        "{candidates:#?}"
    );
    for (parser, payload) in [
        ("Int", "integer"),
        ("Real", "real"),
        ("Bool", "boolean"),
        ("HexInt", "integer"),
    ] {
        let source = format!(
            "program T; uses Std.Parse as Numbers; begin discard Numbers.{parser}(Text := \nend."
        );
        service.documents_mut().close_document(&path);
        service
            .documents_mut()
            .open_document(&path, 1, source.clone())
            .expect("editor buffer");
        let cursor = source.find("Text := ").expect("named argument") + "Text := ".len();
        let help = service
            .signature_help(&path, cursor)
            .expect("signature query")
            .value
            .expect("parser signature");
        assert_eq!(help.signature.parameters, ["Text: string"], "{help:#?}");
        assert_eq!(help.active_parameter, Some(0));
        assert!(
            help.signature
                .label
                .to_ascii_lowercase()
                .ends_with(&format!(": result of ({payload}, parseerror)")),
            "{help:#?}"
        );
    }
}
