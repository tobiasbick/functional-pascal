//! Checked declaration and binding types, without interpreting symbol display strings.

use crate::{DocumentAnalysis, DocumentSymbol, SymbolKind};
use fpas_lexer::Span;
use fpas_sema::{AnalysisMetadata, Ty};

/// Generic parameters are valid checked types; unresolved holes are not.
pub(super) fn usable(ty: &Ty) -> bool {
    match ty {
        Ty::Error | Ty::Named(_) => false,
        Ty::Array(inner) | Ty::Channel(inner) | Ty::Option(inner) | Ty::Task(inner) => {
            usable(inner)
        }
        Ty::Result(left, right) | Ty::Dict(left, right) => usable(left) && usable(right),
        Ty::Function(function) => {
            function.params.iter().all(|p| usable(&p.ty)) && usable(&function.return_type)
        }
        Ty::Procedure(procedure) => procedure.params.iter().all(|p| usable(&p.ty)),
        Ty::Record(record) => record.type_args.iter().all(usable),
        Ty::Enum(enumeration) => enumeration.type_args.iter().all(usable),
        Ty::Distinct(distinct) => usable(&distinct.underlying),
        Ty::Integer
        | Ty::Subrange(_)
        | Ty::Real
        | Ty::Boolean
        | Ty::String
        | Ty::ByteBuffer
        | Ty::Unit
        | Ty::GenericParam(..) => true,
    }
}

pub(super) fn select(
    analysis: &DocumentAnalysis,
    offset: usize,
    consider: &mut impl FnMut(Span, Option<&Ty>),
) {
    let Some(semantic) = analysis.semantic() else {
        return;
    };
    visit(
        analysis.symbols().entries(),
        semantic.metadata(),
        offset,
        None,
        consider,
    );
}

fn visit(
    symbols: &[DocumentSymbol],
    metadata: &AnalysisMetadata,
    offset: usize,
    owner: Option<&Ty>,
    consider: &mut impl FnMut(Span, Option<&Ty>),
) {
    for symbol in symbols {
        let span = symbol.selection_span;
        let nominal = metadata
            .named_types
            .get(&symbol.name.to_ascii_lowercase())
            .or_else(|| {
                metadata
                    .named_types
                    .get(&symbol.qualified_name.to_ascii_lowercase())
            });
        if span.offset() <= offset
            && offset < span.end()
            && matches!(
                symbol.kind,
                SymbolKind::Constant
                    | SymbolKind::Variable
                    | SymbolKind::Parameter
                    | SymbolKind::LoopVariable
                    | SymbolKind::Field
                    | SymbolKind::Function
                    | SymbolKind::Procedure
                    | SymbolKind::Method
            )
        {
            let full = symbol.full_span;
            let key = (full.source_id(), full.offset());
            let ty = metadata
                .declaration_types
                .get(&key)
                .or_else(|| metadata.binding_types.get(&key))
                .or_else(|| metadata.pattern_types.get(&key))
                .or_else(|| member_type(owner, symbol));
            consider(span.into(), ty);
        }
        let child_owner = if matches!(symbol.kind, SymbolKind::Type | SymbolKind::Enum) {
            nominal
        } else {
            owner
        };
        visit(&symbol.children, metadata, offset, child_owner, consider);
    }
}

fn member_type<'a>(owner: Option<&'a Ty>, symbol: &DocumentSymbol) -> Option<&'a Ty> {
    match owner? {
        Ty::Record(record) => record
            .fields
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(&symbol.name))
            .map(|(_, ty)| ty),
        Ty::Enum(enumeration) => {
            let variant = symbol.qualified_name.rsplit('.').nth(1)?;
            enumeration
                .variants
                .iter()
                .find(|v| v.name.eq_ignore_ascii_case(variant))?
                .fields
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case(&symbol.name))
                .map(|(_, ty)| ty)
        }
        _ => None,
    }
}
