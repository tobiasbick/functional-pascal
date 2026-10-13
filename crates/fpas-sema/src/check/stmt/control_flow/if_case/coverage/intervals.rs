//! Exact scalar interval partitions; work is bounded by labels, not domain size.
//! **Documentation:** `docs/pascal/language/pattern-matching/exhaustiveness.md`.

use super::{Checker, Pat, Ty};
use fpas_unit::interface::ConstantValue;
use std::cmp::Ordering;
use std::collections::BTreeSet;

impl Checker {
    /// Representatives of every interval where the set of matching rows is constant.
    pub(super) fn range_values(
        &self,
        rows: &[Vec<Pat>],
        ty: &Ty,
        start: &ConstantValue,
        end: &ConstantValue,
    ) -> Vec<ConstantValue> {
        match (start, end) {
            (ConstantValue::Integer(start), ConstantValue::Integer(end)) => {
                let (start, end) = if let Ty::Subrange(range) = self.resolve_visible_type(ty) {
                    ((*start).max(range.lower), (*end).min(range.upper))
                } else {
                    (*start, *end)
                };
                integer_values(rows, start, end)
            }
            (ConstantValue::String(start), ConstantValue::String(end)) => {
                string_values(rows, start, end)
            }
            _ => self
                .constructors(ty)
                .unwrap_or_default()
                .into_iter()
                .filter_map(|constructor| constructor.value)
                .filter(|value| contains(start, end, value))
                .collect(),
        }
    }

    /// A scalar comparison outside a subrange's bounds matches no possible value.
    pub(super) fn value_in_domain(&self, ty: &Ty, value: &ConstantValue) -> bool {
        match (self.resolve_visible_type(ty), value) {
            (Ty::Subrange(range), ConstantValue::Integer(value)) => range.contains(*value),
            _ => true,
        }
    }
}

fn integer_values(rows: &[Vec<Pat>], start: i64, end: i64) -> Vec<ConstantValue> {
    if start > end {
        return Vec::new();
    }
    let mut points = BTreeSet::from([i128::from(start)]);
    for row in rows {
        let interval = match row.first() {
            Some(Pat::Value(ConstantValue::Integer(value))) => Some((*value, *value)),
            Some(Pat::Range(ConstantValue::Integer(start), ConstantValue::Integer(end))) => {
                Some((*start, *end))
            }
            _ => None,
        };
        if let Some((lower, upper)) = interval {
            for point in [i128::from(lower), i128::from(upper) + 1] {
                if i128::from(start) <= point && point <= i128::from(end) {
                    points.insert(point);
                }
            }
        }
    }
    points
        .into_iter()
        .filter_map(|value| i64::try_from(value).ok())
        .map(ConstantValue::Integer)
        .collect()
}

fn string_values(rows: &[Vec<Pat>], start: &str, end: &str) -> Vec<ConstantValue> {
    if start > end {
        return Vec::new();
    }
    let mut points = BTreeSet::from([start.to_string()]);
    for row in rows {
        let interval = match row.first() {
            Some(Pat::Value(ConstantValue::String(value))) => {
                Some((value.as_str(), value.as_str()))
            }
            Some(Pat::Range(ConstantValue::String(start), ConstantValue::String(end))) => {
                Some((start.as_str(), end.as_str()))
            }
            _ => None,
        };
        if let Some((lower, upper)) = interval {
            // Appending NUL is the immediate lexicographic successor of a UTF-8 string.
            for point in [lower.to_string(), format!("{upper}\0")] {
                if start <= point.as_str() && point.as_str() <= end {
                    points.insert(point);
                }
            }
        }
    }
    points.into_iter().map(ConstantValue::String).collect()
}

/// Compare one typed scalar against both inclusive range endpoints.
pub(super) fn contains(start: &ConstantValue, end: &ConstantValue, value: &ConstantValue) -> bool {
    matches!(
        compare(start, value),
        Some(Ordering::Less | Ordering::Equal)
    ) && matches!(compare(value, end), Some(Ordering::Less | Ordering::Equal))
}

fn compare(left: &ConstantValue, right: &ConstantValue) -> Option<Ordering> {
    match (left, right) {
        (ConstantValue::Integer(left), ConstantValue::Integer(right)) => Some(left.cmp(right)),
        (ConstantValue::String(left), ConstantValue::String(right)) => Some(left.cmp(right)),
        (ConstantValue::Boolean(left), ConstantValue::Boolean(right)) => Some(left.cmp(right)),
        (
            ConstantValue::EnumValue {
                enum_name: left_name,
                backing_value: left,
                ..
            },
            ConstantValue::EnumValue {
                enum_name: right_name,
                backing_value: right,
                ..
            },
        ) if left_name.eq_ignore_ascii_case(right_name) => Some(left.cmp(right)),
        _ => None,
    }
}
