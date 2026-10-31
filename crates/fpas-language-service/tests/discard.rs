//! Statement completion includes the explicit discard keyword.

#![allow(
    clippy::unwrap_used,
    reason = "test fixtures fail fast with direct assertions for diagnostic clarity"
)]

use crate::support;

use fpas_language_service::{CompletionKind, LanguageService, WorkspaceContext};
use support::TempDirectory;

#[test]
fn statement_keywords_are_offered() {
    let temp = TempDirectory::new("discard-completion");
    for (prefix, keyword) in [("disc", "discard"), ("usi", "using")] {
        let source = format!("program T; begin {prefix}; end.");
        let path = temp.write(format!("{keyword}.fpas"), &source);
        let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
        let cursor = source.find(prefix).unwrap() + prefix.len();
        let completions = service.completions(&path, cursor).unwrap().value;
        assert!(completions.iter().any(
            |candidate| candidate.label == keyword && candidate.kind == CompletionKind::Keyword
        ));
    }
}
