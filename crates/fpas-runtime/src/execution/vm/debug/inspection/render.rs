//! Debugger storage origins layered over shared read-only value inspection.

use crate::Executable;

use super::model::DebugInspectionLimits;
use super::targets::MutationAccess;
use crate::execution::vm::inspection::values::{self, ValueLimits};

/// A debugger value retains a stop-local writable origin for mutation commands.
pub(super) type RetainedValue = values::render::RetainedValue<MutationAccess>;
/// Shared presentation with debugger-specific origins on lazy children.
pub(super) type RenderedValue = values::render::RenderedValue<MutationAccess>;

#[cfg(test)]
fn render(value: &RetainedValue, limits: DebugInspectionLimits) -> RenderedValue {
    render_with_executable(value, limits, None)
}

/// Render one bounded variable while keeping its child handles lazy.
pub(super) fn render_with_executable(
    value: &RetainedValue,
    limits: DebugInspectionLimits,
    executable: Option<&Executable>,
) -> RenderedValue {
    let bounds = ValueLimits {
        max_children: limits.max_children,
        max_string_chars: limits.max_string_chars,
        max_summary_chars: limits.max_string_chars.saturating_mul(2).saturating_add(64),
        max_depth: limits.max_depth,
    };
    values::render::render_with_executable(value, bounds, executable)
}

#[cfg(test)]
mod tests;
