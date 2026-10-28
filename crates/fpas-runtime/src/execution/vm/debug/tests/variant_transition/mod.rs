//! Helpers for qualified variant-transition assignment tests.

pub(super) use super::variant_replacement::{
    enum_call, field, fieldless, named, root, scope_reference, stop_with_variants,
    variant_executable,
};
pub(super) use super::*;

pub(super) fn qualified(root_name: &str, fields: &[&str]) -> DebugAssignmentTarget {
    DebugAssignmentTarget {
        root: root_name.to_string(),
        selectors: fields
            .iter()
            .map(|name| DebugAssignmentSelector::Field((*name).to_string()))
            .collect(),
    }
}

mod cases;
