//! Standard editor projection of the shared bounded catalogue.

use crate::{CancellationToken, LanguageService, LanguageServiceError, SymbolLocation};

/// Maximum number of declarations returned by one workspace-symbol query.
pub const WORKSPACE_SYMBOL_LIMIT: usize = 100;

impl LanguageService {
    /// Returns ranked syntax candidates, including empty text and open loose documents.
    pub fn workspace_symbols(
        &mut self,
        query: &str,
    ) -> Result<Vec<SymbolLocation>, LanguageServiceError> {
        Ok(self
            .catalog_declarations(query, WORKSPACE_SYMBOL_LIMIT, &CancellationToken::new())?
            .symbols
            .into_iter()
            .filter_map(|candidate| {
                Some(SymbolLocation {
                    path: candidate.target.snapshot?.path().into(),
                    symbol: candidate.target.symbol?,
                })
            })
            .collect())
    }
}
