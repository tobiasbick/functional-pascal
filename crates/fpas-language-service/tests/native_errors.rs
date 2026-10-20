//! Native error navigation and completion share enum identities on unsaved sources.
#![allow(
    clippy::expect_used,
    reason = "editor fixtures use explicit source offsets"
)]
mod support;

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
