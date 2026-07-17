//! Lowering for event setter assignment (`B.OnClick := …` / `:= nil`).
//!
//! **Documentation:** `docs/pascal/language/types/record-events.md`

use crate::error::{CompileError, internal_compiler_error};
use fpas_bytecode::{Op, SourceLocation, Value};
use fpas_parser::{Designator, Expr};
use fpas_sema::EventWriteInfo;

use super::super::Compiler;

impl Compiler {
    /// Compile an event assignment: receiver once, `Some`/`None` once, call setter.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-events.md`
    pub(in crate::compiler) fn compile_event_assignment(
        &mut self,
        target: &Designator,
        value: &Expr,
        info: &EventWriteInfo,
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        let receiver_parts = target
            .parts
            .get(..info.receiver_part_count)
            .ok_or_else(|| {
                internal_compiler_error(
                    "Event-write receiver metadata exceeds the designator path.",
                    "Re-run compilation and report this internal compiler error.",
                    target.span.line,
                    target.span.column,
                )
            })?;
        let receiver = Designator {
            parts: receiver_parts.to_vec(),
            span: target.span,
        };
        self.compile_property_receiver(&receiver, &info.receiver_reads)?;
        if info.clear {
            self.emit(Op::MakeNone, location);
        } else {
            self.compile_expr(value)?;
            self.emit(Op::MakeSome, location);
        }
        let name_idx = self.add_constant(Value::Str(info.setter_name.clone()), location)?;
        self.emit(Op::Call(name_idx, 2), location);
        self.emit(Op::Pop, location);
        Ok(())
    }
}
