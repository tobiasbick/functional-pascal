//! Postcondition result names are typed symbols confined to their original clause.
//! **Documentation:** `docs/pascal/language/functions/contracts.md`.

use super::{member_symbol, named_type, type_callable_signature, type_text};
use crate::{DocumentSnapshot, DocumentSymbol, SymbolKind};
use fpas_parser::{FunctionDecl, Visibility};

/// Add each named return value with the declaration's return type and its clause scope.
pub(super) fn collect_result_symbols(
    snapshot: &DocumentSnapshot,
    owner: &str,
    declaration: &FunctionDecl,
    output: &mut Vec<DocumentSymbol>,
) {
    for clause in &declaration.ensures {
        let Some((name, span)) = &clause.result_name else {
            continue;
        };
        let mut symbol = member_symbol(
            snapshot,
            owner,
            name,
            SymbolKind::Constant,
            *span,
            Visibility::Private,
            named_type(&declaration.return_type),
            format!(
                "const {name}: {}",
                type_text(snapshot, &declaration.return_type)
            ),
            clause.span.diagnostic_span_or_synthetic(),
            Vec::new(),
        );
        symbol.callable = type_callable_signature(snapshot, name, &declaration.return_type);
        output.push(symbol);
    }
}
