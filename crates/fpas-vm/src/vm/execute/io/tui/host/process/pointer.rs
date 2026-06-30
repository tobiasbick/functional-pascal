//! Pointer fallback dispatch for the hosted `Std.Tui` loop.

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::{SourceLocation, Value};
use fpas_std::{DamageRegion, ProcessOutcome, UiMouse};

use super::DispatchOutcomes;

impl Worker {
    /// Dispatches one mouse event to the registered `OnMouse` handler.
    pub(super) fn dispatch_tui_mouse_event(
        &mut self,
        mouse: UiMouse,
        on_mouse: Option<Value>,
        app_rec: Value,
        line: SourceLocation,
    ) -> Result<ProcessOutcome, VmError> {
        self.dispatch_console_event_handler(
            on_mouse,
            [app_rec, Self::console_mouse_event_record(mouse)],
            Some(DamageRegion::FullFrame),
            DispatchOutcomes {
                hit: ProcessOutcome::Pointer { handled: true },
                miss: ProcessOutcome::Pointer { handled: false },
            },
            line,
        )
    }
}
