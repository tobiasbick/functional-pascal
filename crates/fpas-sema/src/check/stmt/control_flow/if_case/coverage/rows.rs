//! Pattern-matrix column specialization and witness rendering.
//! **Documentation:** `docs/pascal/language/pattern-matching/exhaustiveness.md`.

use super::intervals::contains;
use super::{Pat, Ty};
use fpas_unit::interface::ConstantValue;

/// Expand rows matching a constructor into their payload columns.
pub(super) fn specialize(
    rows: &[Vec<Pat>],
    key: &str,
    arity: usize,
    value: Option<&ConstantValue>,
) -> Vec<Vec<Pat>> {
    rows.iter()
        .filter_map(|row| {
            let (head, rest) = row.split_first()?;
            let mut next = match head {
                Pat::Ctor(head_key, args) if head_key == key && args.len() == arity => args.clone(),
                Pat::Range(start, end)
                    if value.is_some_and(|value| contains(start, end, value)) =>
                {
                    Vec::new()
                }
                Pat::Wild => vec![Pat::Wild; arity],
                _ => return None,
            };
            next.extend_from_slice(rest);
            Some(next)
        })
        .collect()
}

/// Remove a wildcard column while retaining its remaining constraints.
pub(super) fn default_rows(rows: &[Vec<Pat>]) -> Vec<Vec<Pat>> {
    rows.iter()
        .filter_map(|row| match row.split_first() {
            Some((Pat::Wild, rest)) => Some(rest.to_vec()),
            _ => None,
        })
        .collect()
}

/// Keep the tails of rows matching a concrete scalar value.
pub(super) fn value_rows(rows: &[Vec<Pat>], value: &ConstantValue) -> Vec<Vec<Pat>> {
    rows.iter()
        .filter_map(|row| match row.split_first() {
            Some((Pat::Wild, rest)) => Some(rest.to_vec()),
            Some((Pat::Value(previous), rest)) if previous == value => Some(rest.to_vec()),
            Some((Pat::Range(start, end), rest)) if contains(start, end, value) => {
                Some(rest.to_vec())
            }
            Some((Pat::Ctor(key, fields), rest))
                if fields.is_empty() && scalar_key(value).as_deref() == Some(key) =>
            {
                Some(rest.to_vec())
            }
            _ => None,
        })
        .collect()
}

fn scalar_key(value: &ConstantValue) -> Option<String> {
    match value {
        ConstantValue::Boolean(value) => Some(value.to_string()),
        ConstantValue::EnumValue { variant_name, .. } => Some(variant_name.to_ascii_lowercase()),
        _ => None,
    }
}

/// Place constructor payload types before remaining matrix columns.
pub(super) fn concat(first: &[Ty], rest: &[Ty]) -> Vec<Ty> {
    first.iter().chain(rest).cloned().collect()
}

/// Render a missing constructor together with its payload witnesses.
pub(super) fn render(display: &str, fields: &[String]) -> String {
    if fields.is_empty() {
        display.to_string()
    } else {
        format!("{display}({})", fields.join(", "))
    }
}
