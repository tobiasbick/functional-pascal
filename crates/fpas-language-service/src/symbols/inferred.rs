//! Checked local types shared by hover, completion, and navigation symbols.
//!
//! **Documentation:** `docs/pascal/language/basics/local-variables.md`

use fpas_sema::{AnalysisMetadata, Ty};

use super::{CallableSignature, DocumentSymbol, SymbolKind};

/// Enriches inferred local and case binding symbols with their checked receiving types.
pub(super) fn apply_binding_types(symbols: &mut [DocumentSymbol], metadata: &AnalysisMetadata) {
    for symbol in symbols {
        let span = symbol.full_span;
        if let Some(ty) = metadata
            .binding_types
            .get(&(span.source_id(), span.offset()))
            .or_else(|| {
                metadata
                    .pattern_types
                    .get(&(span.source_id(), span.offset()))
            })
            && !ty.is_error()
        {
            let keyword = match symbol.kind {
                SymbolKind::Constant => Some("const"),
                SymbolKind::Variable => Some("var"),
                _ => None,
            };
            if let Some(keyword) = keyword {
                symbol.detail = format!("{keyword} {}: {ty}", symbol.name);
                let callable = match ty {
                    Ty::Function(function) => {
                        Some((&function.params, Some(function.return_type.as_ref())))
                    }
                    Ty::Procedure(procedure) => Some((&procedure.params, None)),
                    _ => None,
                };
                if let Some((params, result)) = callable {
                    let parameters = params
                        .iter()
                        .map(|param| {
                            format!(
                                "{}{}: {}",
                                if param.mode == fpas_parser::ParamMode::Var {
                                    "var "
                                } else {
                                    ""
                                },
                                param.name,
                                param.ty
                            )
                        })
                        .collect::<Vec<_>>();
                    symbol.callable = Some(CallableSignature {
                        label: format!(
                            "{} {}({}){}",
                            if result.is_some() {
                                "function"
                            } else {
                                "procedure"
                            },
                            symbol.name,
                            parameters.join("; "),
                            result.map_or_else(String::new, |ty| format!(": {ty}"))
                        ),
                        parameters,
                    });
                }
                symbol.type_name = match ty {
                    Ty::Record(record) => Some(record.name.clone()),
                    Ty::Enum(enumeration) => Some(enumeration.name.clone()),
                    Ty::Subrange(range) => Some(range.name.clone()),
                    Ty::Distinct(distinct) => Some(match distinct.owner_unit.as_deref() {
                        Some(unit) => {
                            let qualifier = metadata
                                .import_aliases
                                .get(&unit.to_ascii_lowercase())
                                .map_or(unit, String::as_str);
                            format!("{qualifier}.{}", distinct.name)
                        }
                        None => distinct.name.clone(),
                    }),
                    Ty::Named(name) => Some(name.clone()),
                    _ => None,
                };
            }
        }
        apply_binding_types(&mut symbol.children, metadata);
    }
}
