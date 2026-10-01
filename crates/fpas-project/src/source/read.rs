//! Structured source-read failures before a source position is available.

use std::path::Path;

use fpas_diagnostics::{Diagnostic, codes::PROJECT_SOURCE_READ_FAILED};

pub(super) fn read_source(path: &Path) -> Result<Vec<u8>, Diagnostic> {
    std::fs::read(path).map_err(|error| {
        Diagnostic::error_without_source(
            PROJECT_SOURCE_READ_FAILED,
            format!("Error reading source file: {error}"),
            Some("Check that the source file exists and is readable.".into()),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_source_read_has_a_project_code_and_no_position() {
        let diagnostic = read_source(Path::new("")).expect_err("an empty file path cannot be read");
        assert_eq!(diagnostic.code, PROJECT_SOURCE_READ_FAILED);
        assert_eq!(diagnostic.span, None);
        assert_eq!(
            diagnostic.stage(),
            fpas_diagnostics::DiagnosticStage::Project
        );
        let output =
            fpas_diagnostics::render_json(None, None, &diagnostic).expect("diagnostic JSON");
        assert!(output.contains("\"location\":null"));
    }
}
