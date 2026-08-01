//! Navigation from values and aliases to named type declarations.

use std::path::Path;

use super::resolve::{find_type, resolve};
use crate::{LanguageService, LanguageServiceError, NavigationResult, SymbolLocation};

impl LanguageService {
    /// Returns the visible source declaration for the selected symbol's named type.
    pub fn type_definitions(
        &mut self,
        path: &Path,
        offset: usize,
    ) -> Result<NavigationResult<Vec<SymbolLocation>>, LanguageServiceError> {
        let context = self.navigation_context(path)?;
        let value = context
            .target_index
            .and_then(|target_index| {
                resolve(&context.documents, target_index, offset).and_then(
                    |(declaration_index, symbol, _)| {
                        let type_name = symbol.type_name.as_deref()?;
                        let (type_index, type_symbol) = find_type(
                            &context.documents,
                            target_index,
                            declaration_index,
                            type_name,
                        )?;
                        Some(SymbolLocation {
                            path: context.documents[type_index].path.clone(),
                            symbol: type_symbol.clone(),
                        })
                    },
                )
            })
            .into_iter()
            .collect();
        Ok(NavigationResult {
            snapshot: context.snapshot,
            value,
        })
    }
}
