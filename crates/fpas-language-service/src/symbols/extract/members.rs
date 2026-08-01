//! Record and enum member symbols.

use fpas_diagnostics::SourceSpan;
use fpas_parser::{RecordMethod, TypeBody, Visibility};

use super::{
    function_children, function_detail, function_signature, member_symbol, named_type,
    procedure_children, procedure_detail, procedure_signature, type_callable_signature, type_text,
};
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
            let mut children = record
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
            children.extend(
                record
                    .methods
                    .iter()
                    .map(|method| method_symbol(snapshot, owner, method, type_span)),
            );
            children.extend(record.properties.iter().map(|property| {
                let mut symbol = member_symbol(
                    snapshot,
                    owner,
                    &property.name,
                    SymbolKind::Property,
                    property.span,
                    property.visibility,
                    named_type(&property.type_expr),
                    format!(
                        "property {}: {}",
                        property.name,
                        type_text(snapshot, &property.type_expr)
                    ),
                    type_span,
                    Vec::new(),
                );
                symbol.callable =
                    type_callable_signature(snapshot, &property.name, &property.type_expr);
                symbol
            }));
            children.extend(record.events.iter().map(|event| {
                let mut symbol = member_symbol(
                    snapshot,
                    owner,
                    &event.name,
                    SymbolKind::Event,
                    event.span,
                    event.visibility,
                    named_type(&event.type_expr),
                    format!(
                        "event {}: {}",
                        event.name,
                        type_text(snapshot, &event.type_expr)
                    ),
                    type_span,
                    Vec::new(),
                );
                symbol.callable = type_callable_signature(snapshot, &event.name, &event.type_expr);
                symbol
            }));
            children.sort_by_key(|symbol| symbol.full_span.offset);
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

fn method_symbol(
    snapshot: &DocumentSnapshot,
    owner: &str,
    method: &RecordMethod,
    type_span: SourceSpan,
) -> DocumentSymbol {
    match method {
        RecordMethod::Function(value) | RecordMethod::StaticFunction(value) => {
            let qualified = format!("{owner}.{}", value.name);
            let mut symbol = member_symbol(
                snapshot,
                owner,
                &value.name,
                SymbolKind::Function,
                value.span,
                value.visibility,
                named_type(&value.return_type),
                function_detail(snapshot, value),
                type_span,
                function_children(snapshot, &qualified, value),
            );
            let implicit = usize::from(matches!(method, RecordMethod::Function(_)));
            symbol.callable = Some(function_signature(snapshot, value, implicit));
            symbol
        }
        RecordMethod::Procedure(value) | RecordMethod::StaticProcedure(value) => {
            let qualified = format!("{owner}.{}", value.name);
            let mut symbol = member_symbol(
                snapshot,
                owner,
                &value.name,
                SymbolKind::Procedure,
                value.span,
                value.visibility,
                None,
                procedure_detail(snapshot, value),
                type_span,
                procedure_children(snapshot, &qualified, value),
            );
            let implicit = usize::from(matches!(method, RecordMethod::Procedure(_)));
            symbol.callable = Some(procedure_signature(snapshot, value, implicit));
            symbol
        }
    }
}
