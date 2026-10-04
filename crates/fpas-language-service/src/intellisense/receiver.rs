//! Completion candidates for calls that take a value as their first parameter.
//!
//! **Documentation:** `docs/pascal/language/functions/fluent-calls.md`

mod types;

use self::types::accepts;
use crate::navigation::{NavigationDocument, resolve_qualified, resolve_unqualified};
use crate::{DocumentSymbol, SymbolKind};

use super::completion::visible_candidates;

/// Returns visible callables whose first parameter accepts the receiver.
pub(super) fn receiver_callable_candidates<'a>(
    documents: &'a [NavigationDocument],
    target_index: usize,
    receiver: &str,
    offset: usize,
    members: &[(usize, &'a DocumentSymbol)],
) -> Vec<(usize, &'a DocumentSymbol)> {
    let Some(receiver_type) = receiver_type(documents, target_index, receiver, offset) else {
        return Vec::new();
    };
    visible_candidates(documents, target_index, offset)
        .into_iter()
        .filter(|(_, symbol)| {
            !members
                .iter()
                .any(|(_, member)| member.name.eq_ignore_ascii_case(&symbol.name))
                && symbol.callable.as_ref().is_some_and(|signature| {
                    signature.parameters.first().is_some_and(|parameter| {
                        parameter_type(parameter)
                            .is_some_and(|first| accepts(&first, &receiver_type))
                    })
                })
        })
        .collect()
}

fn receiver_type(
    documents: &[NavigationDocument],
    target_index: usize,
    receiver: &str,
    offset: usize,
) -> Option<String> {
    let receiver = receiver.trim();
    if receiver.starts_with('(') && receiver.ends_with(')') {
        return receiver_type(
            documents,
            target_index,
            &receiver[1..receiver.len() - 1],
            offset,
        );
    }
    if receiver.ends_with(')') {
        let open = receiver.find('(')?;
        let name = receiver[..open].trim();
        let parts = name.split('.').map(str::to_owned).collect::<Vec<_>>();
        let (_, symbol) = if parts.len() == 1 {
            resolve_unqualified(documents, target_index, name, offset)?
        } else {
            resolve_qualified(documents, target_index, &parts, offset)?
        };
        return symbol.type_name.or_else(|| {
            symbol
                .detail
                .rsplit_once("): ")
                .map(|(_, ty)| ty.to_owned())
        });
    }
    if receiver.starts_with('\'') && receiver.ends_with('\'') {
        return Some("string".to_owned());
    }
    if receiver.parse::<i64>().is_ok() {
        return Some("integer".to_owned());
    }
    if receiver.parse::<f64>().is_ok() {
        return Some("real".to_owned());
    }
    if receiver.eq_ignore_ascii_case("true") || receiver.eq_ignore_ascii_case("false") {
        return Some("boolean".to_owned());
    }
    let (_, symbol) = resolve_unqualified(documents, target_index, receiver, offset)?;
    if matches!(symbol.kind, SymbolKind::Type | SymbolKind::Enum) {
        return None;
    }
    symbol
        .type_name
        .or_else(|| symbol.detail.rsplit_once(": ").map(|(_, ty)| ty.to_owned()))
}

fn parameter_type(parameter: &str) -> Option<String> {
    parameter
        .split_once(':')
        .map(|(_, ty)| ty.trim().to_owned())
}
