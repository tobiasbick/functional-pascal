//! Deterministic bounded workspace-symbol queries.

use std::cmp::Ordering;

use crate::{LanguageService, LanguageServiceError, SymbolLocation};

/// Maximum number of declarations returned by one workspace-symbol query.
pub const WORKSPACE_SYMBOL_LIMIT: usize = 100;

impl LanguageService {
    /// Searches indexed declarations using case-insensitive short and qualified-name matching.
    pub fn workspace_symbols(
        &mut self,
        query: &str,
    ) -> Result<Vec<SymbolLocation>, LanguageServiceError> {
        let query = query.trim().to_ascii_lowercase();
        let mut matches = self
            .workspace_symbol_index()?
            .all_locations()
            .into_iter()
            .filter_map(|location| match_rank(&location, &query).map(|rank| (rank, location)))
            .collect::<Vec<_>>();
        matches.sort_by(|(left_rank, left), (right_rank, right)| {
            left_rank
                .cmp(right_rank)
                .then_with(|| compare_locations(left, right))
        });
        matches.truncate(WORKSPACE_SYMBOL_LIMIT);
        Ok(matches.into_iter().map(|(_, location)| location).collect())
    }
}

fn match_rank(location: &SymbolLocation, query: &str) -> Option<u8> {
    if query.is_empty() {
        return Some(0);
    }
    let short = location.symbol.name.to_ascii_lowercase();
    let qualified = location.symbol.qualified_name.to_ascii_lowercase();
    if short == query {
        Some(0)
    } else if short.starts_with(query) {
        Some(1)
    } else if short.contains(query) {
        Some(2)
    } else if qualified.starts_with(query) {
        Some(3)
    } else if qualified.contains(query) {
        Some(4)
    } else {
        None
    }
}

fn compare_locations(left: &SymbolLocation, right: &SymbolLocation) -> Ordering {
    left.symbol
        .name
        .to_ascii_lowercase()
        .cmp(&right.symbol.name.to_ascii_lowercase())
        .then_with(|| {
            left.symbol
                .qualified_name
                .to_ascii_lowercase()
                .cmp(&right.symbol.qualified_name.to_ascii_lowercase())
        })
        .then_with(|| left.path.cmp(&right.path))
        .then_with(|| {
            left.symbol
                .selection_span
                .offset()
                .cmp(&right.symbol.selection_span.offset())
        })
}
