//! Instantiated generic record field descriptions.
//! See `docs/pascal/tools/editor-integration.md`.

use super::native_receiver::receiver_type;
use crate::{CallableSignature, DocumentSymbol, SymbolKind, navigation::NavigationDocument};
use fpas_sema::Ty;

/// Retain declaration locations while displaying the receiver's concrete field type.
pub(crate) fn instantiate_member(
    documents: &[NavigationDocument],
    target: usize,
    receiver: &str,
    offset: usize,
    member: &mut DocumentSymbol,
) {
    if member.kind != SymbolKind::Field {
        return;
    }
    let Some(Ty::Record(record)) = receiver_type(documents, target, receiver, offset, 0) else {
        return;
    };
    if record.type_params.is_empty() {
        return;
    }
    let Some((_, ty)) = record
        .fields
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(&member.name))
    else {
        return;
    };
    member.detail = format!("field {}: {ty}", member.name);
    member.callable = callable_signature(&member.name, ty);
    member.type_name = match ty {
        Ty::Record(record) => Some(record.name.clone()),
        Ty::Enum(enumeration) => Some(enumeration.name.clone()),
        Ty::Named(name) => Some(name.clone()),
        _ => None,
    };
}

fn callable_signature(name: &str, ty: &Ty) -> Option<CallableSignature> {
    let (kind, params, result) = match ty {
        Ty::Function(function) => (
            "function",
            &function.params,
            format!(": {}", function.return_type),
        ),
        Ty::Procedure(procedure) => ("procedure", &procedure.params, String::new()),
        _ => return None,
    };
    let parameters = params.iter().map(ToString::to_string).collect::<Vec<_>>();
    Some(CallableSignature {
        label: format!("{kind} {name}({}){result}", parameters.join("; ")),
        parameters,
    })
}
