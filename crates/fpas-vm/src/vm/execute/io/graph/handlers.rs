//! Handler validation helpers for `Std.Graph` intrinsics.
//!
//! **Documentation:** `docs/pascal/std/graph/app/README.md` (from the repository root).

use crate::vm::Worker;
use crate::vm::shared::GraphState;
use fpas_bytecode::SourceLocation;

const GRAPH_APPLICATION_HANDLERS_TYPE: &str = "Std.Graph.ApplicationHandlers";

impl Worker {
    /// Pops a `Std.Graph.ApplicationHandlers` record from the stack.
    pub(super) fn pop_graph_application_handlers(
        &mut self,
        line: SourceLocation,
    ) -> Result<Vec<(String, fpas_bytecode::Value)>, crate::vm::diagnostics::VmError> {
        use crate::vm::diagnostics::TYPE_MISMATCH_CODE;
        use crate::vm::runtime_error;
        use fpas_bytecode::Value;

        match self.pop(line)? {
            Value::Record { type_name, fields } if type_name == GRAPH_APPLICATION_HANDLERS_TYPE => {
                Ok(fields)
            }
            other => Err(runtime_error(
                TYPE_MISMATCH_CODE,
                format!(
                    "Expected {GRAPH_APPLICATION_HANDLERS_TYPE}, got {}",
                    other.type_name()
                ),
                "Pass a `Std.Graph.ApplicationHandlers` record to `Application.Configure(App, Handlers)`.",
                line,
            )),
        }
    }

    /// Acquires the graph state lock for the duration of `f`.
    pub(in crate::vm::execute::io) fn with_graph<R>(
        &self,
        f: impl FnOnce(&mut GraphState) -> R,
    ) -> R {
        f(&mut self.shared.graph.lock().unwrap_or_else(|e| e.into_inner()))
    }

    /// Pops a handler function and an `Application` record, validates arity, then stores it.
    pub(super) fn register_graph_handler(
        &mut self,
        arity: u8,
        label: &'static str,
        hint: &'static str,
        setter: impl FnOnce(&mut GraphState, fpas_bytecode::Value),
        line: SourceLocation,
    ) -> Result<(), crate::vm::diagnostics::VmError> {
        let func = self.pop(line)?;
        self.pop_graph_application(line)?;
        self.validate_host_handler_function(&func, arity, label, hint, line)?;
        self.with_graph(|graph| setter(graph, func));
        Ok(())
    }
}
