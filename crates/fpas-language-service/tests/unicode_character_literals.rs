//! Unsaved Unicode literal edits refresh diagnostics without changing on-disk source.

mod support;

use fpas_diagnostics::codes::LEX_INVALID_CHARACTER_CODE_LITERAL;
use fpas_language_service::{LanguageService, WorkspaceContext};
use support::TempDirectory;

#[test]
fn unicode_character_literal_overlay_diagnostics_follow_edits()
-> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDirectory::new("unicode-literal-overlay");
    let disk = "program T; begin const Text := 'original'; end.";
    let path = temp.write("unicode.fpas", disk);
    let mut service = LanguageService::new(WorkspaceContext::loose(temp.path()));
    for (version, code, invalid) in [
        (1, "#8364", false),
        (2, "#55296", true),
        (3, "#1_114_111", false),
    ] {
        let source = format!("program T; begin const Text := '😀'{code}; end.");
        if version == 1 {
            service
                .documents_mut()
                .open_document(&path, version, source.as_str())?;
        } else {
            service
                .documents_mut()
                .apply_full_text(&path, version, source.as_str())?;
        }
        let analysis = service.analyze_document(&path)?;
        let diagnostics = analysis.diagnostics();
        if invalid {
            assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
            assert_eq!(diagnostics[0].code, LEX_INVALID_CHARACTER_CODE_LITERAL);
            let span = diagnostics[0].span.ok_or("missing character-code span")?;
            assert_eq!(
                span.offset(),
                source.find(code).ok_or("missing source code")?
            );
            assert_eq!(span.length(), code.len());
        } else {
            assert!(diagnostics.is_empty(), "{diagnostics:?}");
        }
    }
    assert_eq!(std::fs::read_to_string(path)?, disk);
    Ok(())
}
