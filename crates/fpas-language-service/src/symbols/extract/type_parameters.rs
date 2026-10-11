//! Shared lexical type-parameter symbols for routines and data types.

use fpas_diagnostics::SourceSpan;
use fpas_parser::TypeParam;

use super::{name_span, source::name_span_after};
use crate::{DocumentSnapshot, DocumentSymbol, SymbolKind, SymbolVisibility};

/// Extracts parameter declarations in source order after the owning name.
pub(super) fn type_parameter_symbols(
    snapshot: &DocumentSnapshot,
    owner: &str,
    parameters: &[TypeParam],
    scope_span: SourceSpan,
) -> Vec<DocumentSymbol> {
    let name = owner.rsplit('.').next().unwrap_or(owner);
    let mut cursor = name_span(snapshot, scope_span, name).end();
    parameters
        .iter()
        .map(|parameter| {
            let selection_span = name_span_after(snapshot, scope_span, &parameter.name, cursor);
            cursor = selection_span.end();
            // Constraint names are keywords, so identifier lookup skips them.
            type_parameter_symbol(owner, parameter, scope_span, selection_span)
        })
        .collect()
}

fn type_parameter_symbol(
    owner: &str,
    parameter: &TypeParam,
    scope_span: SourceSpan,
    selection_span: SourceSpan,
) -> DocumentSymbol {
    let constraint = parameter
        .constraint
        .as_deref()
        .map_or_else(String::new, |value| format!(": {value}"));
    DocumentSymbol {
        name: parameter.name.clone(),
        qualified_name: format!("{owner}.{}", parameter.name),
        kind: SymbolKind::TypeParameter,
        full_span: selection_span,
        selection_span,
        scope_span,
        visible_from: selection_span.offset(),
        visibility: SymbolVisibility::Private,
        type_name: None,
        detail: format!("type parameter {}{constraint}", parameter.name),
        callable: None,
        children: Vec::new(),
    }
}
