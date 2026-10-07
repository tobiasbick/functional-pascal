//! Statement completion includes the explicit discard keyword.

#![allow(
    clippy::unwrap_used,
    reason = "test fixtures fail fast with direct assertions for diagnostic clarity"
)]

mod support;

use fpas_language_service::{CompletionKind, LanguageService, WorkspaceContext};
use support::TempDirectory;

#[test]
fn discard_is_offered_as_a_statement_keyword() {
    let temp = TempDirectory::new("discard-completion");
    let source = "program T; begin disc; end.";
    let path = temp.write("discard.fpas", source);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    let cursor = source.find("disc;").unwrap() + 4;
    let completions = service.completions(&path, cursor).unwrap().value;
    assert!(
        completions
            .iter()
            .any(|candidate| candidate.label == "discard"
                && candidate.kind == CompletionKind::Keyword)
    );
}
