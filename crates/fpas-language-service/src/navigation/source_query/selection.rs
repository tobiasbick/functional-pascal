//! Exact names select only after complete eligible discovery proves uniqueness.

use super::{SourceQueryError, SourceQueryOperation, SourceQueryResult, SourceSession};
use crate::{CancellationToken, SymbolKind};
use std::collections::BTreeSet;

impl SourceSession {
    /// Selects exact short or qualified declaration names, never a partial or first match.
    pub fn query_symbol(
        &mut self,
        name: &str,
        operation: SourceQueryOperation,
        limit: usize,
        cancellation: &CancellationToken,
    ) -> Result<SourceQueryResult, SourceQueryError> {
        let mut found = self.find_declarations(name, true, limit, cancellation)?;
        if found.known_total != Some(1) || !found.reasons.is_empty() {
            let selection_error = match found.known_total {
                Some(0) => Some("no_target"),
                Some(n) if n > 1 => Some("ambiguous_target"),
                _ => None,
            };
            // A completely traversed ambiguous set remains an error even if its display is clipped.
            if selection_error == Some("ambiguous_target") {
                found.reasons.remove("result_limit");
            }
            return Ok(SourceQueryResult {
                calls: None,
                target: None,
                matches: Vec::new(),
                expression_type: None,
                declaration_type: None,
                inspection: None,
                diagnostics: found.diagnostics,
                reasons: found.reasons,
                candidates: found.symbols,
                candidate_total: found.known_total,
                selection_error,
            });
        }
        let Some(candidate) = found.symbols.pop() else {
            return Err(SourceQueryError::new(
                "analysis_unavailable",
                "A unique declaration is missing from the checked result.",
            ));
        };
        let Some((snapshot, symbol)) = candidate
            .target
            .snapshot
            .as_ref()
            .zip(candidate.target.symbol.as_ref())
        else {
            let (calls, call_diagnostics) = if matches!(
                operation,
                SourceQueryOperation::Callers | SourceQueryOperation::Callees
            ) {
                let (calls, diagnostics) =
                    self.call_relations(&candidate.target, operation, limit, cancellation)?;
                (Some(calls), diagnostics)
            } else {
                (None, Vec::new())
            };
            let call_reasons = calls
                .as_ref()
                .map(|c| c.reasons.clone())
                .unwrap_or_default();
            found.diagnostics.extend(call_diagnostics);
            return Ok(SourceQueryResult {
                calls,
                inspection: if operation == SourceQueryOperation::Inspect {
                    Some(
                        self.service()
                            .inspect_target(&candidate.target, cancellation)?,
                    )
                } else {
                    None
                },
                declaration_type: (operation == SourceQueryOperation::Type)
                    .then(|| super::intrinsic::lookup(&candidate.target.qualified_name))
                    .flatten()
                    .filter(|symbol| symbol.kind != fpas_sema::IntrinsicStdSymbolKind::Type)
                    .map(|symbol| super::DeclarationType {
                        display: symbol.ty.to_string(),
                        kind: symbol.ty.category(),
                    }),
                target: Some(candidate.target),
                matches: Vec::new(),
                expression_type: None,
                diagnostics: found.diagnostics,
                reasons: if operation == SourceQueryOperation::References {
                    BTreeSet::from(["unsupported_target"])
                } else {
                    call_reasons
                },
                candidates: Vec::new(),
                candidate_total: Some(1),
                selection_error: None,
            });
        };
        let mut result = self.query_position(
            snapshot.path(),
            symbol.selection_span.offset(),
            operation,
            limit,
            cancellation,
        )?;
        if result.target.as_ref().is_none_or(|target| {
            target.qualified_name != candidate.target.qualified_name
                || target
                    .symbol
                    .as_ref()
                    .is_none_or(|actual| actual.selection_span != symbol.selection_span)
        }) {
            return Err(SourceQueryError::new(
                "analysis_unavailable",
                "Checked discovery and declaration resolution disagree.",
            ));
        }
        if operation == SourceQueryOperation::Type
            && matches!(
                symbol.kind,
                SymbolKind::Unit
                    | SymbolKind::Program
                    | SymbolKind::Type
                    | SymbolKind::Enum
                    | SymbolKind::ImportAlias
            )
        {
            result.expression_type = None;
            result.reasons.remove("type_unavailable");
        }
        result.candidate_total = Some(1);
        Ok(result)
    }
}
