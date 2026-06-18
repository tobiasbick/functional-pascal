//! Hosted `Std.Graph` redraw and paint dispatch.
//!
//! **Documentation:** `docs/pascal/std/graph/app.md` (from the repository root).

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::SourceLocation;

impl Worker {
    /// Consumes a pending redraw, invokes `OnPaint` when registered, then presents the backbuffer.
    ///
    /// Returns `0` = no redraw pending, `5` = `OnPaint` ran, `6` = pending but no handler.
    pub(in crate::vm::execute::io) fn graph_host_dispatch_redraw_inner(
        &mut self,
        line: SourceLocation,
    ) -> Result<i64, VmError> {
        let pending = {
            let graph = self.shared.graph.lock().unwrap_or_else(|e| e.into_inner());
            graph.session.peek_redraw_pending(line)?
        };

        if !pending {
            return Ok(0);
        }

        let on_paint = self.with_graph(|graph| graph.on_paint.clone());
        let app_rec = Self::graph_application_record();

        {
            let mut graph = self.shared.graph.lock().unwrap_or_else(|e| e.into_inner());
            let _ = graph.session.take_redraw_pending(line)?;
        }

        if let Some(handler) = on_paint {
            let _ = self.call_function_sync_allowing_shutdown(
                &handler,
                std::slice::from_ref(&app_rec),
                line,
            )?;
            let mut graph = self.shared.graph.lock().unwrap_or_else(|e| e.into_inner());
            graph.session.present(line)?;
            Ok(5)
        } else {
            Ok(6)
        }
    }
}
