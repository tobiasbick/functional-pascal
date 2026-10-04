//! Editor symbol details for local bindings with semantically inferred types.
//!
//! **Documentation:** `docs/pascal/language/basics/local-variables.md`.

use super::{CallableSignature, DocumentSymbol, DocumentSymbols, SymbolKind};
use fpas_sema::{BindingTypeMap, ParamTy, Ty};

/// Add resolved local binding types without guessing from later source uses.
pub(super) fn apply(symbols: &mut DocumentSymbols, types: &BindingTypeMap) {
    for symbol in symbols.entries_mut() {
        apply_symbol(symbol, types);
    }
}

fn apply_symbol(symbol: &mut DocumentSymbol, types: &BindingTypeMap) {
    if symbol.detail.ends_with(": inferred") {
        let key = (symbol.full_span.source_id(), symbol.full_span.offset());
        if let Some(ty) = types.get(&key).filter(|ty| !ty.is_error()) {
            let keyword = if symbol.kind == SymbolKind::Constant {
                "const"
            } else {
                "var"
            };
            symbol.detail = format!("{keyword} {}: {ty}", symbol.name);
            symbol.type_name = match ty {
                Ty::Record(record) => Some(record.name.clone()),
                Ty::Enum(enumeration) => Some(enumeration.name.clone()),
                Ty::Named(name) => Some(name.clone()),
                _ => None,
            };
            symbol.callable = match ty {
                Ty::Function(function) => Some(signature(
                    "function",
                    &symbol.name,
                    &function.params,
                    &format!(": {}", function.return_type),
                )),
                Ty::Procedure(procedure) => {
                    Some(signature("procedure", &symbol.name, &procedure.params, ""))
                }
                _ => None,
            };
        }
    }
    for child in &mut symbol.children {
        apply_symbol(child, types);
    }
}

fn signature(kind: &str, name: &str, parameters: &[ParamTy], result: &str) -> CallableSignature {
    let parameters = parameters
        .iter()
        .map(|parameter| {
            format!(
                "{}{}: {}",
                if parameter.mutable { "var " } else { "" },
                parameter.name,
                parameter.ty
            )
        })
        .collect::<Vec<_>>();
    CallableSignature {
        label: format!("{kind} {name}({}){result}", parameters.join("; ")),
        parameters,
    }
}
