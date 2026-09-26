//! Completion candidates for calls that take a value as their first parameter.
//!
//! **Documentation:** `docs/pascal/language/functions/fluent-calls.md`

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

fn accepts(expected: &str, actual: &str) -> bool {
    let generic = expected.chars().all(|ch| ch.is_ascii_uppercase())
        || expected.len() == 1 && expected.chars().all(|ch| ch.is_ascii_alphabetic());
    let expected = expected.to_ascii_lowercase();
    let actual = actual.to_ascii_lowercase();
    if expected == actual {
        return true;
    }
    if let Some(expected_inner) = expected.strip_prefix("array of ") {
        return actual
            .strip_prefix("array of ")
            .is_some_and(|actual_inner| accepts(expected_inner, actual_inner));
    }
    if let Some(expected_inner) = expected.strip_prefix("dict of ") {
        return structured_pair(expected_inner, &actual, "dict of ", " to ");
    }
    if let Some(expected_inner) = expected.strip_prefix("result of ") {
        return structured_pair(expected_inner, &actual, "result of ", ", ");
    }
    if expected.starts_with("option of ") {
        return actual
            .strip_prefix("option of ")
            .is_some_and(|inner| accepts(&expected["option of ".len()..], inner));
    }
    if expected.starts_with("channel of ") {
        return actual
            .strip_prefix("channel of ")
            .is_some_and(|inner| accepts(&expected["channel of ".len()..], inner));
    }
    if expected == "task" {
        return actual == "task" || actual.starts_with("task of ");
    }
    if let Some(inner) = expected.strip_prefix("task of ") {
        return actual
            .strip_prefix("task of ")
            .is_some_and(|actual| accepts(inner, actual));
    }
    generic || expected.rsplit('.').next() == actual.rsplit('.').next()
}

fn structured_pair(expected: &str, actual: &str, prefix: &str, separator: &str) -> bool {
    let Some(actual) = actual.strip_prefix(prefix) else {
        return false;
    };
    let Some((left, right)) = expected.split_once(separator) else {
        return false;
    };
    let Some((actual_left, actual_right)) = actual.split_once(separator) else {
        return false;
    };
    accepts(left, actual_left) && accepts(right, actual_right)
}

#[cfg(test)]
mod tests {
    use super::accepts;

    #[test]
    fn first_parameter_shape_filters_collections() {
        assert!(accepts("array of T", "array of integer"));
        assert!(!accepts("array of T", "string"));
        assert!(accepts("dict of K to V", "dict of string to integer"));
        assert!(!accepts("dict of K to V", "array of integer"));
        assert!(!accepts(
            "dict of string to integer",
            "dict of integer to integer"
        ));
        assert!(!accepts("result of string, E", "result of integer, string"));
    }
}
