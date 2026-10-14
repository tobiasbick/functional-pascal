//! Read-only public-signature projection of the current source snapshot.

use std::path::Path;

use fpas_parser::CompilationUnit;

use crate::{LanguageService, LanguageServiceError, NavigationResult};

impl LanguageService {
    /// Returns public signatures from the current unit, including unsaved editor text.
    ///
    /// No compilation or derived-file writes occur. Syntax errors and program files
    /// are rejected instead of presenting a partial export view.
    /// **Documentation:** `docs/pascal/tools/editor-integration.md#public-signatures`
    ///
    /// # Errors
    /// Returns source-loading, syntax, compilation-unit or formatter failures.
    pub fn public_signatures(
        &mut self,
        path: &Path,
    ) -> Result<NavigationResult<String>, LanguageServiceError> {
        let snapshot = self.snapshot(path)?;
        if snapshot.has_parse_errors() {
            return Err(LanguageServiceError::analysis(
                path,
                "Fix the source syntax errors before opening public signatures.",
            ));
        }
        let CompilationUnit::Unit(unit) = snapshot.compilation_unit() else {
            return Err(LanguageServiceError::analysis(
                path,
                "Public signatures are available for unit sources. Open a file beginning with `unit Name;`.",
            ));
        };
        let value = fpas_fmt::format_public_signatures(snapshot.source(), unit)
            .map_err(|error| LanguageServiceError::analysis(path, error.to_string()))?;
        Ok(NavigationResult { snapshot, value })
    }
}
