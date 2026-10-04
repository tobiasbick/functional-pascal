//! Record and enum member symbols.

use fpas_diagnostics::SourceSpan;
use fpas_parser::{TypeBody, Visibility};

use super::{member_symbol, named_type, type_callable_signature, type_text};
use crate::{CallableSignature, DocumentSnapshot, DocumentSymbol, SymbolKind};

pub(super) fn type_children(
    snapshot: &DocumentSnapshot,
    owner: &str,
    body: &TypeBody,
    type_span: SourceSpan,
    declaration_scope: SourceSpan,
) -> Vec<DocumentSymbol> {
    match body {
        TypeBody::Record(record) => {
            let children = record
                .fields
                .iter()
                .map(|field| {
                    let mut symbol = member_symbol(
                        snapshot,
                        owner,
                        &field.name,
                        SymbolKind::Field,
                        field.span,
                        field.visibility,
                        named_type(&field.type_expr),
                        format!(
                            "field {}: {}",
                            field.name,
                            type_text(snapshot, &field.type_expr)
                        ),
                        type_span,
                        Vec::new(),
                    );
                    symbol.callable =
                        type_callable_signature(snapshot, &field.name, &field.type_expr);
                    symbol
                })
                .collect::<Vec<_>>();
            children
        }
        TypeBody::Enum(value) => value
            .members
            .iter()
            .map(|member| {
                let mut symbol = member_symbol(
                    snapshot,
                    owner,
                    &member.name,
                    SymbolKind::EnumMember,
                    member.span,
                    Visibility::Public,
                    Some(owner.to_owned()),
                    format!("enum member {owner}.{}", member.name),
                    declaration_scope,
                    Vec::new(),
                );
                if !member.fields.is_empty() {
                    let parameters = member
                        .fields
                        .iter()
                        .map(|field| {
                            format!("{}: {}", field.name, type_text(snapshot, &field.type_expr))
                        })
                        .collect::<Vec<_>>();
                    symbol.callable = Some(CallableSignature {
                        label: format!("{}({}): {owner}", member.name, parameters.join("; ")),
                        parameters,
                    });
                }
                symbol
            })
            .collect(),
        TypeBody::Alias(_) => Vec::new(),
    }
}
