//! Decode and validate opaque host view handles.

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use crate::vm::runtime_error;
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use fpas_std::ViewId;

impl Worker {
    /// Pops and decodes a required `Std.Tui.ViewId` value.
    pub(in crate::vm::execute::io) fn pop_tui_view_id(
        &mut self,
        line: SourceLocation,
    ) -> Result<ViewId, VmError> {
        let value = self.pop(line)?;
        Self::tui_view_id_from_value(&value, line)
    }

    /// Pops and decodes an optional `Std.Tui.ViewId` value.
    pub(in crate::vm::execute::io) fn pop_optional_tui_view_id(
        &mut self,
        label: &str,
        line: SourceLocation,
    ) -> Result<Option<ViewId>, VmError> {
        match self.pop(line)? {
            Value::OptionNone => Ok(None),
            Value::OptionSome(inner) => Ok(Some(Self::tui_view_id_from_value(&inner, line)?)),
            other => Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!(
                    "{label} expects `Option of ViewId`, got {}",
                    other.type_name()
                ),
                "Pass `None` to detach a view to the root list or `Some(Parent)` to reparent it.",
                line,
            )),
        }
    }

    /// Validates that a host view handle still exists in the registry.
    pub(in crate::vm::execute::io) fn require_registered_tui_view(
        &self,
        view_id: ViewId,
        line: SourceLocation,
    ) -> Result<ViewId, VmError> {
        let exists = self.with_tui(|tui| tui.views.rect(view_id).is_some());
        if exists {
            Ok(view_id)
        } else {
            Err(runtime_error(
                RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                format!("Unknown host view handle {}", view_id.raw()),
                "Pass a `Std.Tui.ViewId` returned by `Application.HostRegisterView(App, X, Y, Width, Height)`.",
                line,
            ))
        }
    }
}
