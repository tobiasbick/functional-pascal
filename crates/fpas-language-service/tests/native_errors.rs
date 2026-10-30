//! Native error navigation and completion share enum identities on unsaved sources.
#![allow(
    clippy::expect_used,
    reason = "editor fixtures use explicit source offsets"
)]
use crate::support;

use fpas_language_service::{LanguageService, SymbolKind, WorkspaceContext};
use support::TempDirectory;

#[test]
fn native_error_completion_survives_incomplete_receiver_expressions() {
    for (declaration, parameter) in [
        ("type Failure = error Failed; end error;", "Value: Failure"),
        ("", "Value: E"),
    ] {
        let temp = TempDirectory::new("incomplete-native-error");
        let source = format!(
            "program Demo; {declaration} function Describe<E: Error>({parameter}): string; begin return Value.\nend function; begin end."
        );
        let path = temp.write("main.fpas", &source);
        let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
        let cursor = source.find("Value.\n").expect("receiver") + "Value.".len();
        let completion = service
            .completions(&path, cursor)
            .expect("completion")
            .value;
        assert!(
            completion.iter().any(|entry| entry.label == "Message"),
            "{parameter}: {completion:?}"
        );
    }
}

#[test]
fn recursive_error_overlays_keep_message_completion_during_incomplete_edits() {
    for (declarations, parameter) in [
        (
            "type Failure = error Failed; Wrapped(Cause: Failure); end error;",
            "Failure",
        ),
        (
            "type Failure = error Failed; Wrapped(Cause: Other); end error; type Other = error Failed; Wrapped(Cause: Failure); end error;",
            "Failure",
        ),
        (
            "type Leaf = error Failed; end error; type Failure of E: Error = error Failed(Cause: E); Wrapped(Cause: Failure of E); end error;",
            "Failure of Leaf",
        ),
    ] {
        let temp = TempDirectory::new("recursive-native-error-overlay");
        let disk = "program Demo; begin end.";
        let path = temp.write("main.fpas", disk);
        let source = format!(
            "program Demo; {declarations} function Describe(Value: {parameter}): string; begin return Value.\nend function; begin end."
        );
        let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
        service
            .documents_mut()
            .open_document(&path, 1, source.as_str())
            .expect("open incomplete error overlay");
        let cursor = source.find("Value.\n").expect("receiver") + "Value.".len();
        let completion = service
            .completions(&path, cursor)
            .expect("completion")
            .value;
        assert!(
            completion
                .iter()
                .any(|entry| entry.label == "Message" && entry.detail.ends_with(": string")),
            "{parameter}: {completion:?}"
        );
        assert_eq!(std::fs::read_to_string(&path).expect("disk source"), disk);
    }
}

#[test]
fn constructor_call_results_keep_message_completion_in_incomplete_overlays() {
    for expression in [
        "Failure.Failed('x')",
        "Make('x')",
        "Leaf.Absent",
        "Wrapped.Failed(Leaf.Absent)",
    ] {
        let temp = TempDirectory::new("constructor-call-error-overlay");
        let disk = "program Demo; begin end.";
        let path = temp.write("main.fpas", disk);
        let source = format!(
            "program Demo; type Failure = error Failed(Text: string); end error; type Leaf = error Absent; end error; type Wrapped of E: Error = error Failed(Cause: E); end error; begin const Make := Failure.Failed; discard {expression}.\nend."
        );
        let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
        service
            .documents_mut()
            .open_document(&path, 1, source.as_str())
            .expect("open incomplete constructor overlay");
        let cursor = source.find(".\n").expect("receiver") + 1;
        let completion = service
            .completions(&path, cursor)
            .expect("completion")
            .value;
        assert!(
            completion
                .iter()
                .any(|entry| entry.label == "Message" && entry.detail.ends_with(": string")),
            "{expression}: {completion:?}"
        );
        assert_eq!(std::fs::read_to_string(&path).expect("disk source"), disk);
    }
}

#[test]
fn constructor_mappers_keep_concrete_result_errors_in_incomplete_overlays() {
    for kind in ["error", "enum"] {
        let temp = TempDirectory::new("constructor-mapper-overlay");
        let disk = "program Demo; begin end.";
        let path = temp.write("main.fpas", disk);
        let source = format!(
            "program Demo; type Failure = {kind} Failed(Text: string); end {kind}; procedure Inspect(Input: result of (integer, string)); begin discard Input.MapError(Failure.Failed).\nend procedure; begin end."
        );
        let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
        service
            .documents_mut()
            .open_document(&path, 1, source.as_str())
            .expect("open incomplete mapper overlay");
        let cursor = source.find(".\n").expect("receiver") + 1;
        let completion = service
            .completions(&path, cursor)
            .expect("completion")
            .value;
        let recovery = completion
            .iter()
            .find(|entry| entry.qualified_name == "Result.OrElse")
            .expect("mapped Result operations");
        assert!(
            recovery.detail.contains("Err: Demo.Failure"),
            "{kind}: {recovery:?}"
        );
        assert_eq!(std::fs::read_to_string(&path).expect("disk source"), disk);
    }
}

#[test]
fn constructor_signature_help_keeps_declared_aliases_in_incomplete_overlays() {
    for kind in ["error", "enum"] {
        let temp = TempDirectory::new("constructor-declared-signature-overlay");
        let disk = "program Demo; begin end.";
        let path = temp.write("main.fpas", disk);
        let source = format!(
            "program Demo; type Text = string; type Failure = {kind} Failed(Value: Text); end {kind}; begin discard Failure.Failed(\nend."
        );
        let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
        service
            .documents_mut()
            .open_document(&path, 1, source.as_str())
            .expect("open incomplete constructor signature overlay");
        let cursor = source.find("Failed(\n").expect("constructor call") + "Failed(".len();
        let help = service
            .signature_help(&path, cursor)
            .expect("constructor signature help")
            .value
            .expect("constructor signature");
        assert_eq!(
            help.signature.parameters,
            ["Value: Text"],
            "{kind}: {help:?}"
        );
        assert_eq!(std::fs::read_to_string(&path).expect("disk source"), disk);
    }
}

#[test]
fn generic_constructor_mappers_keep_applied_result_errors_in_incomplete_overlays() {
    for (declaration, error_type, expected) in [
        (
            "type Wrapped of E: Error = error Failed(Cause: E); end error;",
            "First",
            "Demo.Wrapped of Demo.First",
        ),
        (
            "type Wrapped of E: Error = error Failed(Cause: E); end error;",
            "Second",
            "Demo.Wrapped of Demo.Second",
        ),
        (
            "type Wrapped of T: Numeric = enum Failed(Value: T); end enum;",
            "integer",
            "Demo.Wrapped of integer",
        ),
    ] {
        let temp = TempDirectory::new("generic-constructor-mapper-overlay");
        let disk = "program Demo; begin end.";
        let path = temp.write("main.fpas", disk);
        let source = format!(
            "program Demo; type First = error Item; end error; type Second = error Item; end error; {declaration} procedure Inspect(Input: result of (integer, {error_type})); begin discard Input.MapError(Wrapped.Failed).\nend procedure; begin end."
        );
        let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
        service
            .documents_mut()
            .open_document(&path, 1, source.as_str())
            .expect("open incomplete generic mapper overlay");
        let cursor = source.find(".\n").expect("receiver") + 1;
        let completion = service
            .completions(&path, cursor)
            .expect("completion")
            .value;
        let recovery = completion
            .iter()
            .find(|entry| entry.qualified_name == "Result.OrElse")
            .expect("mapped Result operations");
        assert!(
            recovery.detail.contains(&format!("Err: {expected}")),
            "{error_type}: {recovery:?}"
        );
        assert_eq!(std::fs::read_to_string(&path).expect("disk source"), disk);
    }
}

#[test]
fn generic_constructor_recovery_rejects_invalid_callback_contexts() {
    for (parameters, fields, error_type) in [
        ("E: Error", "Cause: E", "string"),
        ("E: Error", "Cause: E", "Ordinary"),
        ("E: Error", "Cause: E; Step: string", "Leaf"),
        ("(E: Error, Extra: Printable)", "Cause: E", "Leaf"),
    ] {
        let temp = TempDirectory::new("invalid-generic-constructor-mapper-overlay");
        let disk = "program Demo; begin end.";
        let path = temp.write("main.fpas", disk);
        let source = format!(
            "program Demo; type Leaf = error Item; end error; type Ordinary = enum Item; end enum; type Wrapped of {parameters} = error Failed({fields}); end error; procedure Inspect(Input: result of (integer, {error_type})); begin discard Input.MapError(Wrapped.Failed).\nend procedure; begin end."
        );
        let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
        service
            .documents_mut()
            .open_document(&path, 1, source.as_str())
            .expect("open invalid incomplete mapper overlay");
        let cursor = source.find(".\n").expect("receiver") + 1;
        let completion = service
            .completions(&path, cursor)
            .expect("completion")
            .value;
        let recovery = completion
            .iter()
            .find(|entry| entry.qualified_name == "Result.OrElse")
            .expect("Result operations without a fabricated concrete error");
        assert!(
            recovery.detail.contains("Err: E2"),
            "{parameters}, {fields}, {error_type}: {recovery:?}"
        );
        assert_eq!(std::fs::read_to_string(&path).expect("disk source"), disk);
    }
}

#[test]
fn native_error_overlays_keep_symbols_references_rename_and_public_spelling() {
    let temp = TempDirectory::new("native-error-overlay");
    let disk = "unit Demo; end unit;";
    let path = temp.write("errors.fpas", disk);
    let source = r#"unit Demo;
public type FsError = error Missing(Path: string) = 'Missing'; end error;
public type ConfigError = error Failed(Cause: FsError) = 'Cause FsError'; end error;
public function Describe<E: Error>(Value: E): string;
begin return Value.Message(); end function;
end unit;
"#;
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    service
        .documents_mut()
        .open_document(&path, 1, source)
        .expect("open overlay");
    let analysis = service.analyze_document(&path).expect("analyze errors");
    assert!(
        analysis.diagnostics().is_empty(),
        "{:?}",
        analysis.diagnostics()
    );
    let public = service
        .public_signatures(&path)
        .expect("public signatures")
        .value;
    assert!(public.contains("type ConfigError = error"), "{public}");
    assert!(
        public.contains("Failed(Cause: FsError) = 'Cause FsError';"),
        "{public}"
    );
    let declaration = source.find("FsError =").expect("type declaration");
    let usage = source.find("Cause: FsError").expect("cause type") + "Cause: ".len();
    let definitions = service.definitions(&path, usage).expect("definition").value;
    assert_eq!(definitions.len(), 1);
    assert_eq!(definitions[0].symbol.kind, SymbolKind::Enum);
    assert_eq!(definitions[0].symbol.selection_span.offset(), declaration);
    assert_eq!(definitions[0].symbol.detail, "type FsError = error");
    let references = service
        .references(&path, usage, true)
        .expect("references")
        .value;
    assert_eq!(
        references.len(),
        2,
        "message literals create no references: {references:?}"
    );
    let rename = service
        .rename(&path, usage, "StorageError")
        .expect("rename")
        .value;
    assert_eq!(rename.len(), 2);
    let cursor = source.find("Value.Message").expect("native receiver") + "Value.".len();
    let completion = service
        .completions(&path, cursor)
        .expect("completion")
        .value;
    assert!(
        completion
            .iter()
            .any(|entry| entry.label == "Message" && entry.detail.ends_with(": string")),
        "{completion:?}"
    );
    assert_eq!(std::fs::read_to_string(&path).expect("disk source"), disk);
}
