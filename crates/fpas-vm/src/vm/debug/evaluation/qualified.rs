//! Qualified identifier paths used for unresolved constructor designators.

use super::model::DebugExpression;

/// Returns the complete dotted name represented by a field designator.
pub(super) fn field_name(base: &DebugExpression, field: &str) -> Option<String> {
    let mut name = expression_name(base)?;
    name.push('.');
    name.push_str(field);
    Some(name)
}

fn expression_name(expression: &DebugExpression) -> Option<String> {
    match expression {
        DebugExpression::Name(name) => Some(name.clone()),
        DebugExpression::Field { base, name } => field_name(base, name),
        _ => None,
    }
}
