//! Conservative summaries of immutable aggregate ownership edges.
//!
//! These summaries omit only graph-classification work. They say nothing about host resources:
//! a record with integer fields can represent a resource and still be ownership-neutral.

use super::Value;

impl Value {
    /// Whether the graph is known to contain no cells, references or functions.
    ///
    /// Aggregate summaries are constant-time; uniquely owned wrapper chains are unwrapped
    /// iteratively. False is conservative, including for task and opaque resource handles.
    pub(super) fn is_ownership_neutral(&self) -> bool {
        let mut value = self;
        loop {
            match value {
                Self::Integer(_)
                | Self::Real(_)
                | Self::Boolean(_)
                | Self::Str(_)
                | Self::Unit
                | Self::Task(_)
                | Self::OptionNone => return true,
                Self::Array(values) => return values.is_ownership_neutral(),
                Self::Record(value) => return value.is_ownership_neutral(),
                Self::Enum(value) => return value.is_ownership_neutral(),
                Self::Dict(values) => return values.is_ownership_neutral(),
                Self::OptionSome(inner) | Self::ResultOk(inner) | Self::ResultError(inner) => {
                    value = inner;
                }
                Self::Cell(_) | Self::Reference(_) | Self::Function(_) | Self::OpaqueHandle(_) => {
                    return false;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
