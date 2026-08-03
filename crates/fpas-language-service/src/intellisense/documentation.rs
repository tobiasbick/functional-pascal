//! Lazy extraction of declaration documentation.

use std::path::Path;

use crate::{LanguageService, LanguageServiceError};

impl LanguageService {
    /// Resolves contiguous `///` lines for a completion declaration identity.
    pub fn completion_documentation(
        &mut self,
        path: &Path,
        declaration_offset: usize,
        qualified_name: &str,
    ) -> Result<Option<String>, LanguageServiceError> {
        let known = self
            .workspace_symbol_index()?
            .all_locations()
            .into_iter()
            .any(|location| {
                location.path == path
                    && location.symbol.full_span.offset() == declaration_offset
                    && location
                        .symbol
                        .qualified_name
                        .eq_ignore_ascii_case(qualified_name)
            });
        if !known {
            return Ok(None);
        }
        let snapshot = self.snapshot(path)?;
        Ok(preceding_documentation(
            snapshot.source(),
            declaration_offset,
        ))
    }
}

fn preceding_documentation(source: &str, declaration_offset: usize) -> Option<String> {
    let declaration_offset = declaration_offset.min(source.len());
    let line_start = source
        .get(..declaration_offset)?
        .rfind('\n')
        .map_or(0, |offset| offset + 1);
    let before = source.get(..line_start)?;
    let mut lines = before.lines().rev();
    let mut documentation = Vec::new();
    for line in &mut lines {
        let trimmed = line.trim();
        if trimmed.is_empty() && documentation.is_empty() {
            continue;
        }
        let Some(text) = trimmed.strip_prefix("///") else {
            break;
        };
        documentation.push(text.strip_prefix(' ').unwrap_or(text).to_owned());
    }
    documentation.reverse();
    (!documentation.is_empty()).then(|| documentation.join("\n"))
}
